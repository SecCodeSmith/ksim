"""Gymnasium wrappers over the native environments (`pip install ksim[gym]`).

`KsimGymEnv` is a `gymnasium.Env` with a `Box(OBSERVATION_LOW, OBSERVATION_HIGH,
(observation_len,), float32)` observation space and a `MultiDiscrete(action_nvec)` action
space and `metadata = {"render_modes": []}`; actions of any integer dtype are accepted.
`reset(seed=None)` seeds `self.np_random` through `gymnasium.Env.reset` and draws the
simulation seed from it, so a seeded reset is reproducible and later unseeded resets
continue the sequence. `step` returns an empty info dict except on truncation, where
`info["episode_metrics"]` holds the benchmark metrics of the episode
(`KsimEnv.episode_metrics`).

`KsimVectorGymEnv` is a `gymnasium.vector.VectorEnv` over `KsimVectorEnv` with
`metadata["autoreset_mode"] = AutoresetMode.NEXT_STEP`, batched spaces built with
`gymnasium.vector.utils.batch_space`, and seeds drawn like the single env's on `reset`.

`make` and `make_vec` take the scenario path plus `EnvOptions` fields as keyword arguments.
"""

from dataclasses import dataclass
from pathlib import Path
from typing import Any, Literal

import gymnasium
import numpy as np
import numpy.typing as npt

Control = Literal["replicas", "replicas_and_nodes"]
Observation = npt.NDArray[np.float32]
Action = npt.NDArray[np.int64]


@dataclass(frozen=True)
class EnvOptions:
    """Keyword arguments of the native environments; `None` options mean the defaults."""

    max_replica_step: int = 3
    control: Control = "replicas"
    service_slots: int | None = None
    reward_weights: tuple[float, float, float] = (1.0, 0.5, 0.01)


class KsimGymEnv(gymnasium.Env[Observation, Action]):
    def __init__(
        self,
        scenario: str | Path,
        options: EnvOptions | None = None,
    ) -> None:
        raise NotImplementedError

    def reset(
        self, *, seed: int | None = None, options: dict[str, Any] | None = None
    ) -> tuple[Observation, dict[str, Any]]:
        raise NotImplementedError

    def step(self, action: Action) -> tuple[Observation, float, bool, bool, dict[str, Any]]:
        raise NotImplementedError


class KsimVectorGymEnv(gymnasium.vector.VectorEnv[Observation, Action, npt.NDArray[Any]]):
    def __init__(
        self,
        scenario: str | Path,
        num_envs: int,
        options: EnvOptions | None = None,
    ) -> None:
        raise NotImplementedError

    def reset(
        self, *, seed: int | list[int] | None = None, options: dict[str, Any] | None = None
    ) -> tuple[Observation, dict[str, Any]]:
        raise NotImplementedError

    def step(
        self, actions: Action
    ) -> tuple[
        Observation,
        npt.NDArray[np.float32],
        npt.NDArray[np.bool_],
        npt.NDArray[np.bool_],
        dict[str, Any],
    ]:
        raise NotImplementedError


def make(scenario: str | Path, **kwargs: Any) -> KsimGymEnv:
    raise NotImplementedError


def make_vec(scenario: str | Path, num_envs: int, **kwargs: Any) -> KsimVectorGymEnv:
    raise NotImplementedError
