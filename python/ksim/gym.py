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

from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Literal

import gymnasium
import numpy as np
import numpy.typing as npt
from gymnasium.vector.utils import batch_space

from ksim import _native

Control = Literal["replicas", "replicas_and_nodes"]
Observation = npt.NDArray[np.float32]
Action = npt.NDArray[np.integer[Any]]


@dataclass(frozen=True)
class EnvOptions:
    """Keyword arguments of the native environments; `None` options mean the defaults."""

    max_replica_step: int = 3
    control: Control = "replicas"
    service_slots: int | None = None
    reward_weights: tuple[float, float, float] = (1.0, 0.5, 0.01)

    @staticmethod
    def kwargs(options: "EnvOptions | None") -> dict[str, Any]:
        return asdict(options or EnvOptions())


SEED_BOUND = 2**63 - 1


def _as_native_actions(actions: npt.ArrayLike) -> npt.NDArray[np.uint32]:
    values = np.asarray(actions)
    if not np.issubdtype(values.dtype, np.integer):
        raise ValueError(f"actions must be integers, got dtype {values.dtype}")
    if (values < 0).any():
        raise ValueError("actions must be non-negative")
    return np.asarray(values, dtype=np.uint32)


def _draw_seed(generator: np.random.Generator) -> int:
    return int(generator.integers(0, SEED_BOUND))


def _observation_space(observation_len: int) -> gymnasium.spaces.Box:
    return gymnasium.spaces.Box(
        _native.OBSERVATION_LOW,
        _native.OBSERVATION_HIGH,
        (observation_len,),
        dtype=np.float32,
    )


def _action_space(action_nvec: list[int]) -> gymnasium.spaces.MultiDiscrete:
    return gymnasium.spaces.MultiDiscrete(action_nvec)


class KsimGymEnv(gymnasium.Env[Observation, Action]):
    metadata: dict[str, Any] = {"render_modes": []}  # noqa: RUF012

    def __init__(self, scenario: str | Path, options: EnvOptions | None = None) -> None:
        self._native = _native.KsimEnv(str(scenario), **EnvOptions.kwargs(options))
        self.observation_space = _observation_space(self._native.observation_len)
        self.action_space = _action_space(self._native.action_nvec)

    @property
    def episode_steps(self) -> int:
        return self._native.episode_steps

    def reset(
        self, *, seed: int | None = None, options: dict[str, Any] | None = None
    ) -> tuple[Observation, dict[str, Any]]:
        super().reset(seed=seed)
        return self._native.reset(_draw_seed(self.np_random)), {}

    def step(self, action: Action) -> tuple[Observation, float, bool, bool, dict[str, Any]]:
        observation, reward, terminated, truncated = self._native.step(_as_native_actions(action))
        info = {"episode_metrics": self._native.episode_metrics()} if truncated else {}
        return observation, reward, terminated, truncated, info


class KsimVectorGymEnv(gymnasium.vector.VectorEnv[Observation, Action, npt.NDArray[Any]]):
    metadata: dict[str, Any] = {  # noqa: RUF012
        "render_modes": [],
        "autoreset_mode": gymnasium.vector.AutoresetMode.NEXT_STEP,
    }

    def __init__(
        self, scenario: str | Path, num_envs: int, options: EnvOptions | None = None
    ) -> None:
        self._native = _native.KsimVectorEnv(str(scenario), num_envs, **EnvOptions.kwargs(options))
        self.num_envs = num_envs
        self.single_observation_space = _observation_space(self._native.observation_len)
        self.single_action_space = _action_space(self._native.action_nvec)
        self.observation_space = batch_space(self.single_observation_space, num_envs)
        self.action_space = batch_space(self.single_action_space, num_envs)

    def reset(
        self, *, seed: int | list[int] | None = None, options: dict[str, Any] | None = None
    ) -> tuple[Observation, dict[str, Any]]:
        super().reset(seed=self._base_seed(seed))
        return self._native.reset(_draw_seed(self.np_random)), {}

    def step(
        self, actions: Action
    ) -> tuple[
        Observation,
        npt.NDArray[np.float32],
        npt.NDArray[np.bool_],
        npt.NDArray[np.bool_],
        dict[str, Any],
    ]:
        observations, rewards, terminated, truncated = self._native.step(
            _as_native_actions(actions)
        )
        return observations, rewards, terminated, truncated, {}

    def _base_seed(self, seed: int | list[int] | None) -> int | None:
        if not isinstance(seed, list):
            return seed
        if seed != list(range(seed[0], seed[0] + self.num_envs)):
            raise ValueError("per-env seeds must be consecutive: env i is seeded with seed + i")
        return seed[0]


def make(scenario: str | Path, **kwargs: Any) -> KsimGymEnv:
    return KsimGymEnv(scenario, EnvOptions(**kwargs))


def make_vec(scenario: str | Path, num_envs: int, **kwargs: Any) -> KsimVectorGymEnv:
    return KsimVectorGymEnv(scenario, num_envs, EnvOptions(**kwargs))
