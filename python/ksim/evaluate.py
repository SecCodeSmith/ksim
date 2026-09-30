"""Benchmark a trained policy with the same seeds and metrics as `ksim-bench run`, so it can be
ranked by `python -m ksim.analysis` next to the baselines.

Each seed runs one full episode on the native `KsimEnv`, reset with exactly that seed (not a
seed drawn by Gymnasium), stepping `policy(observation)` until truncation; the report is
`{"scenario": <scenario TOML name>, "policy": policy_name, "seed": seed,
"metrics": env.episode_metrics()}`. The default seeds `range(10)` match
`ksim-bench run --seeds 10`.
"""

import json
import tomllib
from collections.abc import Callable, Sequence
from dataclasses import asdict
from pathlib import Path
from typing import Any

import numpy as np
import numpy.typing as npt

from ksim import _native
from ksim.gym import EnvOptions, _as_native_actions

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
    if len(seeds) == 0:
        raise ValueError("evaluate needs at least one seed")
    name = _scenario_name(scenario)
    env = _native.KsimEnv(str(scenario), **asdict(options or EnvOptions()))
    return [
        {
            "scenario": name,
            "policy": policy_name,
            "seed": seed,
            "metrics": _run_episode(env, policy, seed),
        }
        for seed in seeds
    ]


def write_reports(reports: list[dict[str, Any]], out: Path) -> Path:
    """Writes `<scenario>-<policy>.jsonl` (one JSON object per line) into `out`, creating it,
    and returns the file path; the name comes from the first report. Raises ValueError for no
    reports or reports that disagree on scenario or policy."""
    if not reports:
        raise ValueError("write_reports needs at least one report")
    scenario, policy = reports[0]["scenario"], reports[0]["policy"]
    if any(r["scenario"] != scenario or r["policy"] != policy for r in reports):
        raise ValueError("reports must share one scenario and one policy")
    out.mkdir(parents=True, exist_ok=True)
    path = out / f"{scenario}-{policy}.jsonl"
    path.write_text("".join(json.dumps(report) + "\n" for report in reports))
    return path


def _scenario_name(scenario: Path) -> str:
    with open(scenario, "rb") as file:
        return str(tomllib.load(file)["name"])


def _run_episode(env: _native.KsimEnv, policy: Policy, seed: int) -> dict[str, float]:
    observation = env.reset(seed)
    truncated = False
    while not truncated:
        action = _as_native_actions(policy(observation))
        observation, _, _, truncated = env.step(action)
    return env.episode_metrics()
