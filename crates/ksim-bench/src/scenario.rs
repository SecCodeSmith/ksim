use ksim_core::{
    cluster::{NodePoolSpec, ServiceSpec},
    engine::{Simulation, SimulationBuilder},
    ids::ServiceId,
    performance::PerformanceModel,
    resources::{MIB, Resources},
    time::{SimDuration, SimTime},
    workload::WorkloadSource,
};
use ksim_metrics::metric::Slo;
use ksim_perf::{queueing::QueueingModel, utilization::UtilizationModel};
use ksim_workload::synthetic::{
    Constant, CurveWorkload, DemandCurve, DemandProfile, FlashCrowd, Ramp, Sinusoid, Step,
};
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum ScenarioError {
    #[error("invalid scenario TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("scenario is inconsistent: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PoolScenario {
    pub name: String,
    pub cpu_millis: u64,
    pub memory_mib: u64,
    pub min_nodes: u32,
    pub max_nodes: u32,
    pub initial_nodes: u32,
    pub price_per_hour: f64,
}

/// Demand curve, mirroring `ksim_workload::synthetic` (times in seconds).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CurveScenario {
    Constant {
        rate: f64,
    },
    Step {
        before: f64,
        after: f64,
        at_s: u64,
    },
    Ramp {
        from: f64,
        to: f64,
        start_s: u64,
        end_s: u64,
    },
    Sinusoid {
        mean: f64,
        amplitude: f64,
        period_s: u64,
        noise_sd: f64,
    },
    FlashCrowd {
        base: f64,
        peak: f64,
        start_s: u64,
        decay_s: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PerformanceScenario {
    Queueing {
        cpu_millis_per_request: f64,
        timeout_ms: f64,
        ca2: f64,
        cs2: f64,
    },
    Utilization {
        base_latency_ms: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HpaScenario {
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_utilization: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceScenario {
    pub name: String,
    pub requests_cpu_millis: u64,
    pub requests_memory_mib: u64,
    pub limits_cpu_millis: u64,
    pub limits_memory_mib: u64,
    pub initial_replicas: u32,
    pub cpu_millis_per_request: f64,
    pub memory_base_mib: f64,
    pub memory_mib_per_rps: f64,
    pub curve: CurveScenario,
    pub performance: PerformanceScenario,
    pub hpa: HpaScenario,
}

/// A benchmark scenario file. Durations are seconds; the workload seed of noisy curves is the
/// episode seed so every seed is a different but reproducible realisation.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub name: String,
    pub duration_s: u64,
    pub tick_s: u64,
    pub slo: Slo,
    pub pools: Vec<PoolScenario>,
    pub services: Vec<ServiceScenario>,
}

impl Scenario {
    /// # Errors
    /// Malformed TOML, or no pools, no services, a zero tick, or a duration shorter than a tick.
    pub fn from_toml_str(text: &str) -> Result<Self, ScenarioError> {
        let scenario: Self = toml::from_str(text)?;
        scenario.validate()?;
        Ok(scenario)
    }

    fn validate(&self) -> Result<(), ScenarioError> {
        let problem = if self.pools.is_empty() {
            Some("no pools")
        } else if self.services.is_empty() {
            Some("no services")
        } else if self.tick_s == 0 {
            Some("tick_s must be positive")
        } else if self.duration_s < self.tick_s {
            Some("duration_s is shorter than one tick")
        } else {
            None
        };
        problem.map_or(Ok(()), |reason| {
            Err(ScenarioError::Invalid(reason.to_owned()))
        })
    }

    #[must_use]
    pub fn steps(&self) -> u64 {
        self.duration_s / self.tick_s
    }

    #[must_use]
    pub fn tick(&self) -> SimDuration {
        SimDuration::from_secs(self.tick_s)
    }

    /// The scenario's true demand for `seed`, also used to drive the simulation.
    #[must_use]
    pub fn workload(&self, seed: u64) -> Box<dyn WorkloadSource> {
        let services = self
            .services
            .iter()
            .map(|service| (service.curve.build(seed), service.demand_profile()))
            .collect();
        Box::new(CurveWorkload::new(services))
    }

    pub fn service_ids(&self) -> impl Iterator<Item = ServiceId> {
        (0..).map(ServiceId).take(self.services.len())
    }

    /// # Errors
    /// Propagates engine build errors as [`ScenarioError::Invalid`].
    pub fn build_simulation(&self, seed: u64) -> Result<Simulation, ScenarioError> {
        let builder = SimulationBuilder::new(seed)
            .tick(self.tick())
            .workload(self.workload(seed));
        let builder = self
            .pools
            .iter()
            .fold(builder, |builder, pool| builder.pool(pool.spec()));
        let builder =
            self.services
                .iter()
                .zip(self.service_ids())
                .fold(builder, |builder, (service, id)| {
                    builder
                        .service(service.spec())
                        .performance_for(id, service.performance.build())
                });
        builder
            .build()
            .map_err(|error| ScenarioError::Invalid(format!("{error:?}")))
    }
}

fn mib_to_bytes(mib: u64) -> u64 {
    mib.saturating_mul(MIB)
}

#[allow(clippy::cast_precision_loss)]
fn mib_to_bytes_f64(mib: f64) -> f64 {
    mib * MIB as f64
}

impl PoolScenario {
    fn spec(&self) -> NodePoolSpec {
        NodePoolSpec {
            name: self.name.clone(),
            allocatable: Resources::new(self.cpu_millis, mib_to_bytes(self.memory_mib)),
            min_nodes: self.min_nodes,
            max_nodes: self.max_nodes,
            initial_nodes: self.initial_nodes,
            price_per_hour: self.price_per_hour,
        }
    }
}

impl ServiceScenario {
    fn spec(&self) -> ServiceSpec {
        ServiceSpec {
            name: self.name.clone(),
            requests: Resources::new(
                self.requests_cpu_millis,
                mib_to_bytes(self.requests_memory_mib),
            ),
            limits: Resources::new(self.limits_cpu_millis, mib_to_bytes(self.limits_memory_mib)),
            initial_replicas: self.initial_replicas,
        }
    }

    fn demand_profile(&self) -> DemandProfile {
        DemandProfile {
            cpu_millis_per_request: self.cpu_millis_per_request,
            memory_base_bytes: mib_to_bytes_f64(self.memory_base_mib),
            memory_bytes_per_rps: mib_to_bytes_f64(self.memory_mib_per_rps),
        }
    }
}

impl CurveScenario {
    fn build(&self, seed: u64) -> Box<dyn DemandCurve> {
        match *self {
            Self::Constant { rate } => Box::new(Constant { rate }),
            Self::Step {
                before,
                after,
                at_s,
            } => Box::new(Step {
                before,
                after,
                at: SimTime::from_secs(at_s),
            }),
            Self::Ramp {
                from,
                to,
                start_s,
                end_s,
            } => Box::new(Ramp {
                from,
                to,
                start: SimTime::from_secs(start_s),
                end: SimTime::from_secs(end_s),
            }),
            Self::Sinusoid {
                mean,
                amplitude,
                period_s,
                noise_sd,
            } => Box::new(Sinusoid {
                mean,
                amplitude,
                period: SimDuration::from_secs(period_s),
                noise_sd,
                seed,
            }),
            Self::FlashCrowd {
                base,
                peak,
                start_s,
                decay_s,
            } => Box::new(FlashCrowd {
                base,
                peak,
                start: SimTime::from_secs(start_s),
                decay: SimDuration::from_secs(decay_s),
            }),
        }
    }
}

impl PerformanceScenario {
    fn build(&self) -> Box<dyn PerformanceModel> {
        match *self {
            Self::Queueing {
                cpu_millis_per_request,
                timeout_ms,
                ca2,
                cs2,
            } => Box::new(QueueingModel::allen_cunneen(
                cpu_millis_per_request,
                timeout_ms,
                ca2,
                cs2,
            )),
            Self::Utilization { base_latency_ms } => Box::new(UtilizationModel { base_latency_ms }),
        }
    }
}
