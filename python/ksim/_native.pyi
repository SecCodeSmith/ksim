from typing import Literal

import numpy as np
import numpy.typing as npt

def native_version() -> str: ...

OBSERVATION_LOW: float
OBSERVATION_HIGH: float

class KsimEnv:
    """One simulated cluster over `ksim_env::Env` (see the Rust docs for the observation and
    action layout). Errors from the environment (bad scenario, wrong action length or value,
    step before reset or after truncation) raise ValueError. Stepping releases the GIL, but
    instances are not thread-safe: use them from the thread that created them."""

    def __init__(
        self,
        scenario_path: str,
        *,
        max_replica_step: int = 3,
        control: Literal["replicas", "replicas_and_nodes"] = "replicas",
        service_slots: int | None = None,
        reward_weights: tuple[float, float, float] = (1.0, 0.5, 0.01),
    ) -> None: ...
    @property
    def observation_len(self) -> int: ...
    @property
    def action_nvec(self) -> list[int]: ...
    @property
    def episode_steps(self) -> int: ...
    def reset(self, seed: int) -> npt.NDArray[np.float32]: ...
    def step(
        self, action: npt.NDArray[np.uint32]
    ) -> tuple[npt.NDArray[np.float32], float, bool, bool]:
        """Returns (observation, reward, terminated, truncated)."""
    def episode_metrics(self) -> dict[str, float]:
        """Metrics of the ticks since the last reset; empty before the first reset."""

class KsimVectorEnv:
    """`num_envs` environments stepped in parallel with Gymnasium `NEXT_STEP` autoreset;
    observations are (num_envs, observation_len), actions (num_envs, len(action_nvec)).
    Instances are not thread-safe: use them from the thread that created them."""

    def __init__(
        self,
        scenario_path: str,
        num_envs: int,
        *,
        max_replica_step: int = 3,
        control: Literal["replicas", "replicas_and_nodes"] = "replicas",
        service_slots: int | None = None,
        reward_weights: tuple[float, float, float] = (1.0, 0.5, 0.01),
    ) -> None: ...
    @property
    def num_envs(self) -> int: ...
    @property
    def observation_len(self) -> int: ...
    @property
    def action_nvec(self) -> list[int]: ...
    def reset(self, seed: int) -> npt.NDArray[np.float32]: ...
    def step(
        self, actions: npt.NDArray[np.uint32]
    ) -> tuple[
        npt.NDArray[np.float32],
        npt.NDArray[np.float32],
        npt.NDArray[np.bool_],
        npt.NDArray[np.bool_],
    ]:
        """Returns (observations, rewards, terminated, truncated)."""
