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

import json
import math
import os
from dataclasses import dataclass, fields
from pathlib import Path

import numpy as np

from ksim.datasets.ksimtrace import ServiceSeries, read

_MIB = 2**20
_MEMORY_STEP_MIB = 64
_MIN_MAX_REPLICAS = 10
_SIZING_PERCENTILE = 95


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


@dataclass(frozen=True)
class _ServiceSizing:
    name: str
    requests_memory_mib: int
    initial_replicas: int
    max_replicas: int
    cpu_millis_per_request: float


def render(trace: Path, output: Path, options: ScenarioOptions) -> str:
    """The scenario TOML text for `trace` as it will be saved at `output`; the name defaults to
    the trace file stem. Raises ValueError for an unreadable trace or a non-positive option."""
    return _render_with_service_count(trace, output, options)[0]


def generate(trace: Path, output: Path, options: ScenarioOptions) -> int:
    """Write `render(...)` to `output`, creating parent directories; returns the service count."""
    text, service_count = _render_with_service_count(trace, output, options)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(text, encoding="utf-8")
    return service_count


def _render_with_service_count(
    trace: Path, output: Path, options: ScenarioOptions
) -> tuple[str, int]:
    _validate(options)
    tick_s, series = _read_trace(trace)
    sizings = [_size_service(item, options) for item in series]
    duration_s = options.duration_s or len(series[0].cpu_millis) * tick_s
    sections = [
        _header(options.name or trace.stem, duration_s, tick_s, options),
        _trace_section(_relative_posix(trace, output)),
        _pool_section(sizings, options),
        *(_service_section(sizing, options) for sizing in sizings),
    ]
    return "\n".join(sections), len(sizings)


def _validate(options: ScenarioOptions) -> None:
    for field in fields(options):
        value = getattr(options, field.name)
        if field.name != "name" and value is not None and value <= 0:
            raise ValueError(f"{field.name} must be positive, got {value}")


def _read_trace(trace: Path) -> tuple[int, list[ServiceSeries]]:
    try:
        return read(trace)
    except OSError as error:
        raise ValueError(f"cannot read trace {trace}: {error}") from error


def _relative_posix(trace: Path, output: Path) -> str:
    relative = os.path.relpath(trace.resolve(), output.resolve().parent)
    return Path(relative).as_posix()


def _size_service(series: ServiceSeries, options: ScenarioOptions) -> _ServiceSizing:
    cpu = series.cpu_millis.astype(np.float64)
    capacity = options.requests_cpu_millis * options.hpa_target
    max_replicas = max(_MIN_MAX_REPLICAS, math.ceil(2 * float(cpu.max()) / capacity))
    p95_cpu = float(np.percentile(cpu, _SIZING_PERCENTILE))
    initial_replicas = min(max(1, math.ceil(p95_cpu / capacity)), max_replicas)
    p95_memory_mib = float(np.percentile(series.memory_bytes, _SIZING_PERCENTILE)) / _MIB
    steps = math.ceil(p95_memory_mib / initial_replicas / _MEMORY_STEP_MIB)
    return _ServiceSizing(
        name=series.name,
        requests_memory_mib=max(1, steps) * _MEMORY_STEP_MIB,
        initial_replicas=initial_replicas,
        max_replicas=max_replicas,
        cpu_millis_per_request=_cpu_millis_per_request(series, options),
    )


def _cpu_millis_per_request(series: ServiceSeries, options: ScenarioOptions) -> float:
    rate = series.request_rate.astype(np.float64)
    busy = rate > 0
    if not busy.any():
        return options.cpu_millis_per_request
    return float(np.median(series.cpu_millis.astype(np.float64)[busy] / rate[busy]))


def _pool_node_count(sizings: list[_ServiceSizing], options: ScenarioOptions) -> int:
    demand = sum(sizing.initial_replicas for sizing in sizings) * options.requests_cpu_millis
    return min(max(1, math.ceil(demand / options.pool_cpu_millis)), options.max_nodes)


def _string(value: str) -> str:
    return json.dumps(value, ensure_ascii=False).replace("\x7f", "\\u007f")


def _header(name: str, duration_s: int, tick_s: int, options: ScenarioOptions) -> str:
    return (
        f"name = {_string(name)}\n"
        f"duration_s = {duration_s}\n"
        f"tick_s = {tick_s}\n"
        "\n[slo]\n"
        f"latency_ms = {options.slo_latency_ms!r}\n"
        'percentile = "P95"\n'
        "max_error_rate = 0.01\n"
        "availability_target = 0.999\n"
    )


def _trace_section(path: str) -> str:
    return f"[trace]\npath = {_string(path)}\nwrap = true\n"


def _pool_section(sizings: list[_ServiceSizing], options: ScenarioOptions) -> str:
    return (
        "[[pools]]\n"
        'name = "default"\n'
        f"cpu_millis = {options.pool_cpu_millis}\n"
        f"memory_mib = {options.pool_memory_mib}\n"
        "min_nodes = 1\n"
        f"max_nodes = {options.max_nodes}\n"
        f"initial_nodes = {_pool_node_count(sizings, options)}\n"
        f"price_per_hour = {options.price_per_hour!r}\n"
    )


def _service_section(sizing: _ServiceSizing, options: ScenarioOptions) -> str:
    per_request = repr(sizing.cpu_millis_per_request)
    return (
        "[[services]]\n"
        f"name = {_string(sizing.name)}\n"
        f"requests_cpu_millis = {options.requests_cpu_millis}\n"
        f"requests_memory_mib = {sizing.requests_memory_mib}\n"
        f"limits_cpu_millis = {2 * options.requests_cpu_millis}\n"
        f"limits_memory_mib = {2 * sizing.requests_memory_mib}\n"
        f"initial_replicas = {sizing.initial_replicas}\n"
        f"cpu_millis_per_request = {per_request}\n"
        f"memory_base_mib = {sizing.requests_memory_mib / 2!r}\n"
        "memory_mib_per_rps = 0.0\n"
        f'curve = {{ kind = "trace", service = {_string(sizing.name)} }}\n'
        f'performance = {{ kind = "queueing", cpu_millis_per_request = {per_request}, '
        "timeout_ms = 10000.0, ca2 = 1.0, cs2 = 1.0 }\n"
        f"hpa = {{ min_replicas = 1, max_replicas = {sizing.max_replicas}, "
        f"target_cpu_utilization = {options.hpa_target!r} }}\n"
    )
