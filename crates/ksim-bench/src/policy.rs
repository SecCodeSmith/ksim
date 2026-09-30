use std::fmt;
use std::str::FromStr;

use ksim_autoscalers::{
    ca::{ClusterAutoscaler, ClusterAutoscalerConfig, Expander},
    hpa::{Hpa, HpaBehavior, HpaSpec},
    reference::{Oracle, StaticReplicas},
    stack::KubernetesStack,
    traits::{NodeProvisioner, ReplicaRecommender, ResourceRecommender},
    vpa::{Vpa, VpaConfig},
};
use ksim_core::{
    autoscaler::Autoscaler,
    time::{SimDuration, SimTime},
};

use crate::scenario::Scenario;

const ORACLE_STARTUP_ALLOWANCE_MILLIS: u64 = 5_000;

/// Autoscaling policies the runner can evaluate. `K8s` is HPA + Cluster Autoscaler (the
/// default stack); `K8sFull` adds the VPA, which upstream warns interferes with a CPU HPA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Policy {
    Hpa,
    Vpa,
    Ca,
    K8s,
    K8sFull,
    StaticPeak,
    StaticAverage,
    Oracle,
}

impl Policy {
    pub const ALL: [Policy; 8] = [
        Policy::Hpa,
        Policy::Vpa,
        Policy::Ca,
        Policy::K8s,
        Policy::K8sFull,
        Policy::StaticPeak,
        Policy::StaticAverage,
        Policy::Oracle,
    ];

    /// Kebab-case name used on the command line and in reports.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Policy::Hpa => "hpa",
            Policy::Vpa => "vpa",
            Policy::Ca => "ca",
            Policy::K8s => "k8s",
            Policy::K8sFull => "k8s-full",
            Policy::StaticPeak => "static-peak",
            Policy::StaticAverage => "static-average",
            Policy::Oracle => "oracle",
        }
    }

    /// Static policies size from the scenario's demand sampled every tick for `seed`; the
    /// oracle looks ahead by two ticks plus the default pod startup maximum (5 s), because a
    /// replica change made at `t` is scheduled at `t + tick`, runs after the startup latency and
    /// is first observed at `t + 2 * tick`; the target
    /// utilisation of reference policies is each service's HPA target. Reference policies only
    /// choose replicas; like `K8s`, they run with the Cluster Autoscaler so their pods get
    /// nodes, which keeps the comparison about replica decisions alone.
    #[must_use]
    pub fn build(self, scenario: &Scenario, seed: u64) -> Box<dyn Autoscaler> {
        match self {
            Policy::Hpa => Box::new(KubernetesStack::new().with_replicas(hpa(scenario))),
            Policy::Vpa => Box::new(KubernetesStack::new().with_resources(vpa(scenario))),
            Policy::Ca => Box::new(KubernetesStack::new().with_nodes(cluster_autoscaler(seed))),
            Policy::K8s => Box::new(
                KubernetesStack::new()
                    .with_replicas(hpa(scenario))
                    .with_nodes(cluster_autoscaler(seed)),
            ),
            Policy::K8sFull => Box::new(
                KubernetesStack::new()
                    .with_replicas(hpa(scenario))
                    .with_resources(vpa(scenario))
                    .with_nodes(cluster_autoscaler(seed)),
            ),
            Policy::StaticPeak => with_cluster_autoscaler(
                Box::new(StaticReplicas::peak(
                    &demand_samples(scenario, seed),
                    &cpu_requests(scenario),
                    target_utilization(scenario),
                )),
                seed,
            ),
            Policy::StaticAverage => with_cluster_autoscaler(
                Box::new(StaticReplicas::average(
                    &demand_samples(scenario, seed),
                    &cpu_requests(scenario),
                    target_utilization(scenario),
                )),
                seed,
            ),
            Policy::Oracle => with_cluster_autoscaler(
                Box::new(Oracle::new(
                    scenario.workload(seed),
                    cpu_requests(scenario),
                    target_utilization(scenario),
                    SimDuration::from_millis(
                        2 * scenario.tick().as_millis() + ORACLE_STARTUP_ALLOWANCE_MILLIS,
                    ),
                    scenario.tick(),
                )),
                seed,
            ),
        }
    }
}

fn with_cluster_autoscaler(
    replicas: Box<dyn ReplicaRecommender>,
    seed: u64,
) -> Box<dyn Autoscaler> {
    Box::new(
        KubernetesStack::new()
            .with_replicas(replicas)
            .with_nodes(cluster_autoscaler(seed)),
    )
}

impl fmt::Display for Policy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown policy {0:?}")]
pub struct UnknownPolicy(pub String);

impl FromStr for Policy {
    type Err = UnknownPolicy;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Policy::ALL
            .into_iter()
            .find(|policy| policy.name() == text)
            .ok_or_else(|| UnknownPolicy(text.to_owned()))
    }
}

fn hpa(scenario: &Scenario) -> Box<dyn ReplicaRecommender> {
    let specs = scenario
        .services
        .iter()
        .map(|service| {
            Some(HpaSpec {
                min_replicas: service.hpa.min_replicas,
                max_replicas: service.hpa.max_replicas,
                target_cpu_utilization: service.hpa.target_cpu_utilization,
                behavior: HpaBehavior::default(),
            })
        })
        .collect();
    Box::new(Hpa::new(specs))
}

fn vpa(scenario: &Scenario) -> Box<dyn ResourceRecommender> {
    Box::new(Vpa::new(
        VpaConfig::default(),
        vec![true; scenario.services.len()],
    ))
}

/// The Cluster Autoscaler of the `k8s` policy, for callers that manage nodes themselves.
#[must_use]
pub fn cluster_autoscaler(seed: u64) -> Box<dyn NodeProvisioner> {
    Box::new(ClusterAutoscaler::new(ClusterAutoscalerConfig {
        expander: Expander::LeastWaste,
        seed,
        ..ClusterAutoscalerConfig::default()
    }))
}

fn cpu_requests(scenario: &Scenario) -> Vec<u64> {
    scenario
        .services
        .iter()
        .map(|service| service.requests_cpu_millis)
        .collect()
}

fn target_utilization(scenario: &Scenario) -> f64 {
    scenario
        .services
        .first()
        .map_or(1.0, |service| service.hpa.target_cpu_utilization)
}

fn demand_samples(scenario: &Scenario, seed: u64) -> Vec<Vec<f64>> {
    let workload = scenario.workload(seed);
    scenario
        .service_ids()
        .map(|service| {
            (0..scenario.steps())
                .map(|step| {
                    let now = SimTime::from_secs(step * scenario.tick_s);
                    workload.demand(service, now).cpu_millis
                })
                .collect()
        })
        .collect()
}
