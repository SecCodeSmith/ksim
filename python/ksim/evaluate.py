"""Benchmark a trained policy with the same seeds and metrics as `ksim-bench run`, so it can be
ranked by `python -m ksim.analysis` next to the baselines.

Each seed runs one full episode on the native `KsimEnv`, reset with exactly that seed (not a
seed drawn by Gymnasium), stepping `policy(observation)` until truncation; the report is
`{"scenario": <scenario TOML name>, "policy": policy_name, "seed": seed,
"metrics": env.episode_metrics()}`. The default seeds `range(10)` match
`ksim-bench run --seeds 10`.
"""

from collections.abc import Callable, Sequence
from pathlib import Path
from typing import Any

import numpy as np
import numpy.typing as npt

from ksim.gym import EnvOptions

Policy = Callable[[npt.NDArray[np.float32]], npt.ArrayLike]


def evaluate(
    scenario: Path,
    policy: Policy,
    policy_name: str,
    *,
    seeds: Sequence[int] = range(10),
    options: EnvOptions | None = None,
) -> list[dict[str, Any]]:
    """One report per seed, in the given order. Raises ValueError for no seeds."""
    raise NotImplementedError


def write_reports(reports: list[dict[str, Any]], out: Path) -> Path:
    """Writes `<scenario>-<policy>.jsonl` (one JSON object per line) into `out`, creating it,
    and returns the file path; the name comes from the first report. Raises ValueError for no
    reports or reports that disagree on scenario or policy."""
    raise NotImplementedError
