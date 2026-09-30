"""Generate a benchmark scenario TOML for a `.ksimtrace` file, one service per trace series.

Sizing rules, per trace series (cpu in millicores, memory in bytes, all per tick):
- `requests_cpu_millis` = `options.requests_cpu_millis`; limits are twice the requests.
- `cpu_millis_per_request` = median of `cpu_millis / request_rate` over ticks with a positive
  rate, or `options.cpu_millis_per_request` when no tick has one; the queueing performance
  model uses the same value (`timeout_ms = 10000`, `ca2 = cs2 = 1`).
- capacity per replica = `requests_cpu_millis * hpa_target`;
  `initial_replicas` = ceil(P95(cpu_millis) / capacity) clamped to [1, max_replicas];
  `max_replicas` = max(10, ceil(2 * max(cpu_millis) / capacity)); `min_replicas` = 1.
- `requests_memory_mib` = P95(memory_bytes) / 2**20 / initial_replicas rounded up to a multiple
  of 64 MiB, at least 64; limits are twice the requests; `memory_base_mib` is half the
  requests and `memory_mib_per_rps` is 0.
- the single pool `default` starts with ceil(sum(initial_replicas * requests_cpu_millis) /
  pool_cpu_millis) nodes clamped to [1, max_nodes]; `min_nodes` = 1.
- `[trace]` points at the trace relative to the output file's directory, `wrap = true`;
  `duration_s` defaults to the trace length and `tick_s` is the trace tick.
- the SLO is P95 latency `slo_latency_ms`, `max_error_rate = 0.01`,
  `availability_target = 0.999`.
"""

from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class ScenarioOptions:
    name: str | None = None
    duration_s: int | None = None
    requests_cpu_millis: int = 500
    cpu_millis_per_request: float = 50.0
    hpa_target: float = 0.6
    pool_cpu_millis: int = 8000
    pool_memory_mib: int = 32768
    max_nodes: int = 50
    price_per_hour: float = 0.3
    slo_latency_ms: float = 250.0


def render(trace: Path, output: Path, options: ScenarioOptions) -> str:
    """The scenario TOML text for `trace` as it will be saved at `output`; the name defaults to
    the trace file stem. Raises ValueError for an unreadable trace or a non-positive option."""
    raise NotImplementedError


def generate(trace: Path, output: Path, options: ScenarioOptions) -> int:
    """Write `render(...)` to `output`, creating parent directories; returns the service count."""
    raise NotImplementedError
