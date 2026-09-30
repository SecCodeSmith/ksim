"""k6 load scripts that replay a demand series with the `ramping-arrival-rate` executor."""

import json
from collections.abc import Sequence
from dataclasses import dataclass


@dataclass(frozen=True)
class Stage:
    duration_s: int
    target_rps: int


def stages(demand_rps: Sequence[float], tick_s: int, rate_scale: float = 1.0) -> list[Stage]:
    """One stage per tick: `duration_s = tick_s`, `target_rps = round(demand * rate_scale)`,
    never negative. Raises ValueError for an empty demand, a non-positive tick or a negative
    scale."""
    if not demand_rps:
        raise ValueError("demand must not be empty")
    if tick_s <= 0:
        raise ValueError("tick_s must be positive")
    if rate_scale < 0:
        raise ValueError("rate_scale must not be negative")
    return [Stage(tick_s, max(0, round(demand * rate_scale))) for demand in demand_rps]


def render_script(stages: Sequence[Stage], url: str, max_vus: int = 200) -> str:
    """A k6 JavaScript module: `import http from 'k6/http'`; `export const options` with one
    scenario `replay` using `executor: 'ramping-arrival-rate'`, `timeUnit: '1s'`,
    `startRate` equal to the first stage's target, `preAllocatedVUs` equal to `max_vus // 4`
    (at least 1), `maxVUs` equal to `max_vus`, and `stages` as
    `{ duration: '<n>s', target: <rps> }` in order; `export default function` issuing
    `http.get(url)`. Raises ValueError for no stages or `max_vus < 1`."""
    if not stages:
        raise ValueError("at least one stage is required")
    if max_vus < 1:
        raise ValueError("max_vus must be at least 1")
    stage_lines = ",\n".join(
        f"        {{ duration: '{stage.duration_s}s', target: {stage.target_rps} }}"
        for stage in stages
    )
    return _SCRIPT_TEMPLATE.format(
        start_rate=stages[0].target_rps,
        pre_allocated_vus=max(1, max_vus // 4),
        max_vus=max_vus,
        stages=stage_lines,
        url=json.dumps(url),
    )


_SCRIPT_TEMPLATE = """import http from 'k6/http';

export const options = {{
  scenarios: {{
    replay: {{
      executor: 'ramping-arrival-rate',
      timeUnit: '1s',
      startRate: {start_rate},
      preAllocatedVUs: {pre_allocated_vus},
      maxVUs: {max_vus},
      stages: [
{stages},
      ],
    }},
  }},
}};

export default function () {{
  http.get({url});
}}
"""
