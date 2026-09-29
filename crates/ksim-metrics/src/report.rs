use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{metric::Metric, trace::EpisodeTrace};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeReport {
    pub scenario: String,
    pub policy: String,
    pub seed: u64,
    pub metrics: BTreeMap<String, f64>,
}

/// Computes every metric on the trace, keyed by `Metric::name`.
#[must_use]
pub fn evaluate(trace: &EpisodeTrace, metrics: &[Box<dyn Metric>]) -> BTreeMap<String, f64> {
    let _ = (trace, metrics);
    todo!()
}
