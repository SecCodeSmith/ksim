"""Typing and dtype contracts of the Gymnasium wrappers under gymnasium 1.4 generic spaces."""

import subprocess
import sys
from pathlib import Path

import gymnasium
import ksim._native as native
import numpy as np
from ksim.gym import KsimGymEnv, KsimVectorGymEnv

REPO_ROOT = Path(__file__).resolve().parents[2]
FIXTURES_DIR = Path(__file__).parent / "fixtures"
TINY_SCENARIO = str(FIXTURES_DIR / "scenarios" / "tiny.toml")


def test_gym_module_passes_strict_mypy():
    result = subprocess.run(
        [sys.executable, "-m", "mypy", "python/ksim/gym.py"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, f"mypy reported errors:\n{result.stdout}{result.stderr}"


def test_observation_space_is_float32_box():
    env = KsimGymEnv(TINY_SCENARIO)
    space = env.observation_space
    assert isinstance(space, gymnasium.spaces.Box)
    assert space.dtype == np.float32
    assert space.low.dtype == np.float32
    assert space.high.dtype == np.float32
    assert np.all(space.low == native.OBSERVATION_LOW)
    assert np.all(space.high == native.OBSERVATION_HIGH)


def test_vector_observation_space_is_float32_box():
    env = KsimVectorGymEnv(TINY_SCENARIO, 2)
    observation_len = KsimGymEnv(TINY_SCENARIO).observation_space.shape[0]
    for space in (env.single_observation_space, env.observation_space):
        assert isinstance(space, gymnasium.spaces.Box)
        assert space.dtype == np.float32
        assert space.low.dtype == np.float32
        assert space.high.dtype == np.float32
        assert np.all(space.low == native.OBSERVATION_LOW)
        assert np.all(space.high == native.OBSERVATION_HIGH)
    assert env.single_observation_space.shape == (observation_len,)
    assert env.observation_space.shape == (2, observation_len)
