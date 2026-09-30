"""k6 load scripts that replay a demand series with the `ramping-arrival-rate` executor."""

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
    raise NotImplementedError


def render_script(stages: Sequence[Stage], url: str, max_vus: int = 200) -> str:
    """A k6 JavaScript module: `import http from 'k6/http'`; `export const options` with one
    scenario `replay` using `executor: 'ramping-arrival-rate'`, `timeUnit: '1s'`,
    `startRate` equal to the first stage's target, `preAllocatedVUs` equal to `max_vus // 4`
    (at least 1), `maxVUs` equal to `max_vus`, and `stages` as
    `{ duration: '<n>s', target: <rps> }` in order; `export default function` issuing
    `http.get(url)`. Raises ValueError for no stages or `max_vus < 1`."""
    raise NotImplementedError
