use std::fmt;
use std::str::FromStr;

use ksim_core::autoscaler::Autoscaler;

use crate::scenario::Scenario;

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
        todo!()
    }

    /// Static policies size from the scenario's demand sampled every tick for `seed`; the
    /// oracle looks ahead by one tick plus the default pod startup maximum (5 s); the target
    /// utilisation of reference policies is each service's HPA target.
    #[must_use]
    pub fn build(self, scenario: &Scenario, seed: u64) -> Box<dyn Autoscaler> {
        let _ = (scenario, seed);
        todo!()
    }
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
        let _ = text;
        todo!()
    }
}
