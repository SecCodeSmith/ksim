use ksim_core::{
    performance::{PerformanceModel, ServiceCapacity, ServiceSignals},
    workload::ServiceDemand,
};

/// Erlang C probability that an arrival waits in an M/M/c queue with offered load
/// `a = lambda / mu`; returns 1 when `a >= servers` (unstable queue) or `servers == 0`.
#[must_use]
pub fn erlang_c(servers: u32, offered_load: f64) -> f64 {
    if servers == 0 || offered_load >= f64::from(servers) {
        return 1.0;
    }
    let erlang_b = (1..=servers).fold(1.0, |b, n| {
        let weighted = offered_load * b;
        weighted / (f64::from(n) + weighted)
    });
    let c = f64::from(servers);
    c * erlang_b / (c - offered_load * (1.0 - erlang_b))
}

/// Waiting-time quantile in seconds of a stable M/M/c queue, from
/// `P(Wq > t) = C(c, a) * exp(-(c*mu - lambda) * t)`; zero when `C(c, a) <= 1 - p`.
#[must_use]
pub fn mmc_wait_quantile(servers: u32, lambda: f64, mu: f64, p: f64) -> f64 {
    let wait_probability = erlang_c(servers, lambda / mu);
    let tail = 1.0 - p;
    if wait_probability <= tail {
        return 0.0;
    }
    let spare_rate = f64::from(servers) * mu - lambda;
    (wait_probability / tail).ln() / spare_rate
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
        Self::allen_cunneen(cpu_millis_per_request, timeout_ms, 1.0, 1.0)
    }

    #[must_use]
    pub fn allen_cunneen(cpu_millis_per_request: f64, timeout_ms: f64, ca2: f64, cs2: f64) -> Self {
        Self {
            cpu_millis_per_request,
            timeout_ms,
            ca2,
            cs2,
        }
    }
}

impl PerformanceModel for QueueingModel {
    fn evaluate(&self, capacity: &ServiceCapacity, demand: &ServiceDemand) -> ServiceSignals {
        if capacity.ready_pods == 0 {
            return self.all_requests_fail(demand);
        }
        let queue = self.queue_for(capacity, demand.request_rate);
        let timed_out_share = self.timed_out_share(&queue);
        let dropped = demand.request_rate - queue.admitted;
        let error_rate = if demand.request_rate > 0.0 {
            (dropped + queue.admitted * timed_out_share) / demand.request_rate
        } else {
            0.0
        };
        ServiceSignals {
            cpu_usage_millis: queue.admitted * self.cpu_millis_per_request,
            memory_usage_bytes: demand.memory_bytes,
            served_rps: queue.admitted,
            latency_p50_ms: self.latency_quantile_ms(&queue, 0.50),
            latency_p95_ms: self.latency_quantile_ms(&queue, 0.95),
            latency_p99_ms: self.latency_quantile_ms(&queue, 0.99),
            error_rate,
            throttled_fraction: 0.0,
            oom: false,
        }
    }
}

/// Utilisation above which arrivals are shed instead of queued.
const MAX_UTILISATION: f64 = 0.99;

/// A stable M/M/c station: `servers` workers of rate `mu` fed by `admitted` arrivals per second.
struct Station {
    servers: u32,
    mu: f64,
    admitted: f64,
}

impl QueueingModel {
    fn queue_for(&self, capacity: &ServiceCapacity, arrival_rate: f64) -> Station {
        let mu = crate::quantity_as_f64(capacity.limits.cpu_millis) / self.cpu_millis_per_request;
        let servers = capacity.ready_pods;
        let admitted = arrival_rate.min(MAX_UTILISATION * f64::from(servers) * mu);
        Station {
            servers,
            mu,
            admitted,
        }
    }

    fn variability_factor(&self) -> f64 {
        f64::midpoint(self.ca2, self.cs2)
    }

    fn timeout_secs(&self) -> f64 {
        self.timeout_ms / 1000.0
    }

    fn timed_out_share(&self, station: &Station) -> f64 {
        let spare_rate = f64::from(station.servers) * station.mu - station.admitted;
        let slack = self.timeout_secs() - 1.0 / station.mu;
        let wait_probability = erlang_c(station.servers, station.admitted / station.mu);
        let k = self.variability_factor().max(f64::MIN_POSITIVE);
        (wait_probability * (-spare_rate * slack / k).exp()).clamp(0.0, 1.0)
    }

    fn latency_quantile_ms(&self, station: &Station, p: f64) -> f64 {
        let wait = mmc_wait_quantile(station.servers, station.admitted, station.mu, p);
        let seconds = self.variability_factor() * wait + 1.0 / station.mu;
        (seconds * 1000.0).min(self.timeout_ms)
    }

    fn all_requests_fail(&self, demand: &ServiceDemand) -> ServiceSignals {
        ServiceSignals {
            memory_usage_bytes: demand.memory_bytes,
            latency_p50_ms: self.timeout_ms,
            latency_p95_ms: self.timeout_ms,
            latency_p99_ms: self.timeout_ms,
            error_rate: if demand.request_rate > 0.0 { 1.0 } else { 0.0 },
            ..ServiceSignals::default()
        }
    }
}
