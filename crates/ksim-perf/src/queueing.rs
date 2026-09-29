use ksim_core::{
    performance::{PerformanceModel, ServiceCapacity, ServiceSignals},
    workload::ServiceDemand,
};

/// Erlang C probability that an arrival waits in an M/M/c queue with offered load
/// `a = lambda / mu`; returns 1 when `a >= servers` (unstable queue) or `servers == 0`.
#[must_use]
pub fn erlang_c(servers: u32, offered_load: f64) -> f64 {
    let _ = (servers, offered_load);
    todo!()
}

/// Waiting-time quantile in seconds of a stable M/M/c queue, from
/// `P(Wq > t) = C(c, a) * exp(-(c*mu - lambda) * t)`; zero when `C(c, a) <= 1 - p`.
#[must_use]
pub fn mmc_wait_quantile(servers: u32, lambda: f64, mu: f64, p: f64) -> f64 {
    let _ = (servers, lambda, mu, p);
    todo!()
}

/// Each ready pod serves `mu = limits.cpu_millis / cpu_millis_per_request` requests per second.
/// Waits follow M/M/c scaled by the Allen-Cunneen factor `(ca2 + cs2) / 2`; latency quantile =
/// scaled wait quantile + `1 / mu`, each capped at `timeout_ms`. Arrivals above 99% utilisation
/// are dropped and counted as errors; of the served requests, the fraction whose response
/// exceeds the timeout, `C * exp(-(c*mu - lambda) * (timeout - 1/mu) / k)` with
/// `k = (ca2 + cs2) / 2`, times out and counts as errors too. `served_rps` is the admitted
/// rate. No ready pods: every request errors, latencies equal `timeout_ms`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QueueingModel {
    pub cpu_millis_per_request: f64,
    pub timeout_ms: f64,
    pub ca2: f64,
    pub cs2: f64,
}

impl QueueingModel {
    #[must_use]
    pub fn mmc(cpu_millis_per_request: f64, timeout_ms: f64) -> Self {
        let _ = (cpu_millis_per_request, timeout_ms);
        todo!()
    }

    #[must_use]
    pub fn allen_cunneen(cpu_millis_per_request: f64, timeout_ms: f64, ca2: f64, cs2: f64) -> Self {
        let _ = (cpu_millis_per_request, timeout_ms, ca2, cs2);
        todo!()
    }
}

impl PerformanceModel for QueueingModel {
    fn evaluate(&self, capacity: &ServiceCapacity, demand: &ServiceDemand) -> ServiceSignals {
        let _ = (capacity, demand);
        todo!()
    }
}
