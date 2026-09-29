use ksim_core::engine::Simulation;
use ksim_core::workload::WorkloadSource;
use ksim_metrics::metric::Slo;
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
        let _ = text;
        todo!()
    }

    #[must_use]
    pub fn steps(&self) -> u64 {
        todo!()
    }

    /// The scenario's true demand for `seed`, also used to drive the simulation.
    #[must_use]
    pub fn workload(&self, seed: u64) -> Box<dyn WorkloadSource> {
        let _ = seed;
        todo!()
    }

    /// # Errors
    /// Propagates engine build errors as [`ScenarioError::Invalid`].
    pub fn build_simulation(&self, seed: u64) -> Result<Simulation, ScenarioError> {
        let _ = seed;
        todo!()
    }
}
