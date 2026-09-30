"""Tests for the native Rust environment bindings (KsimEnv, KsimVectorEnv)."""

from pathlib import Path

import ksim._native as native
import numpy as np
import pytest

FIXTURES_DIR = Path(__file__).parent / "fixtures"
TINY_SCENARIO = str(FIXTURES_DIR / "scenarios" / "tiny.toml")


class TestKsimEnvProperties:
    """Test basic properties of KsimEnv."""

    def test_observation_len(self):
        """observation_len returns the observation length: service_slots * 10 + pools * 5 + 2."""
        env = native.KsimEnv(TINY_SCENARIO)
        obs_len = env.observation_len
        assert isinstance(obs_len, int)
        assert obs_len > 0

    def test_action_nvec(self):
        """action_nvec returns list of action dimensions."""
        env = native.KsimEnv(TINY_SCENARIO)
        nvec = env.action_nvec
        assert isinstance(nvec, list)
        assert all(isinstance(n, int) and n > 0 for n in nvec)

    def test_episode_steps_is_10(self):
        """episode_steps returns 10 for duration_s=150, tick_s=15."""
        env = native.KsimEnv(TINY_SCENARIO)
        steps = env.episode_steps
        assert steps == 10

    def test_episode_steps_is_10_with_max_replica_step(self):
        """episode_steps is unchanged by max_replica_step."""
        env1 = native.KsimEnv(TINY_SCENARIO)
        env2 = native.KsimEnv(TINY_SCENARIO, max_replica_step=1)
        assert env1.episode_steps == env2.episode_steps == 10


class TestKsimEnvReset:
    """Test reset behavior."""

    def test_reset_returns_float32_array(self):
        """reset returns float32 array of observation_len."""
        env = native.KsimEnv(TINY_SCENARIO)
        obs = env.reset(seed=0)
        assert isinstance(obs, np.ndarray)
        assert obs.dtype == np.float32
        assert len(obs) == env.observation_len

    def test_reset_returns_observations_within_bounds(self):
        """reset returns observations within [OBSERVATION_LOW, OBSERVATION_HIGH]."""
        env = native.KsimEnv(TINY_SCENARIO)
        obs = env.reset(seed=0)
        assert np.all(obs >= native.OBSERVATION_LOW)
        assert np.all(obs <= native.OBSERVATION_HIGH)

    def test_reset_multiple_times_with_different_seeds(self):
        """Resetting with different seeds produces different observations."""
        env = native.KsimEnv(TINY_SCENARIO)
        obs1 = env.reset(seed=0)
        obs2 = env.reset(seed=1)
        assert not np.allclose(obs1, obs2)

    def test_reset_deterministic_with_same_seed(self):
        """Resetting with the same seed produces the same observation."""
        env1 = native.KsimEnv(TINY_SCENARIO)
        env2 = native.KsimEnv(TINY_SCENARIO)
        obs1 = env1.reset(seed=42)
        obs2 = env2.reset(seed=42)
        np.testing.assert_array_equal(obs1, obs2)


class TestKsimEnvStep:
    """Test step behavior."""

    def test_step_returns_tuple_of_four(self):
        """step returns (obs, reward, terminated, truncated)."""
        env = native.KsimEnv(TINY_SCENARIO)
        env.reset(seed=0)
        action = np.zeros(len(env.action_nvec), dtype=np.uint32)
        result = env.step(action)
        assert isinstance(result, tuple)
        assert len(result) == 4

    def test_step_returns_correct_types(self):
        """step returns (float32 array, float, bool, bool)."""
        env = native.KsimEnv(TINY_SCENARIO)
        env.reset(seed=0)
        action = np.zeros(len(env.action_nvec), dtype=np.uint32)
        obs, reward, terminated, truncated = env.step(action)
        assert isinstance(obs, np.ndarray)
        assert obs.dtype == np.float32
        assert len(obs) == env.observation_len
        assert isinstance(reward, float)
        assert isinstance(terminated, bool)
        assert isinstance(truncated, bool)

    def test_observation_within_bounds_after_step(self):
        """step returns observations within [OBSERVATION_LOW, OBSERVATION_HIGH]."""
        env = native.KsimEnv(TINY_SCENARIO)
        env.reset(seed=0)
        action = np.zeros(len(env.action_nvec), dtype=np.uint32)
        obs, _, _, _ = env.step(action)
        assert np.all(obs >= native.OBSERVATION_LOW)
        assert np.all(obs <= native.OBSERVATION_HIGH)

    def test_step_before_reset_raises_value_error(self):
        """Stepping before reset raises ValueError."""
        env = native.KsimEnv(TINY_SCENARIO)
        action = np.zeros(len(env.action_nvec), dtype=np.uint32)
        with pytest.raises(ValueError):
            env.step(action)

    def test_step_with_wrong_action_length_raises_value_error(self):
        """Step with wrong action length raises ValueError."""
        env = native.KsimEnv(TINY_SCENARIO)
        env.reset(seed=0)
        action = np.zeros(len(env.action_nvec) + 1, dtype=np.uint32)
        with pytest.raises(ValueError):
            env.step(action)

    def test_step_with_out_of_range_action_raises_value_error(self):
        """Step with out-of-range action raises ValueError."""
        env = native.KsimEnv(TINY_SCENARIO)
        env.reset(seed=0)
        action = np.zeros(len(env.action_nvec), dtype=np.uint32)
        # Set an action value beyond its nvec range
        action[0] = env.action_nvec[0] + 100
        with pytest.raises(ValueError):
            env.step(action)


class TestKsimEnvTruncation:
    """Test truncation behavior."""

    def test_step_truncates_after_episode_steps(self):
        """Step returns truncated=True after episode_steps steps."""
        env = native.KsimEnv(TINY_SCENARIO)
        env.reset(seed=0)
        action = np.zeros(len(env.action_nvec), dtype=np.uint32)
        for _ in range(env.episode_steps - 1):
            _, _, _, truncated = env.step(action)
            assert not truncated
        _, _, _, truncated = env.step(action)
        assert truncated

    def test_step_after_truncation_raises_value_error(self):
        """Stepping after truncation raises ValueError."""
        env = native.KsimEnv(TINY_SCENARIO)
        env.reset(seed=0)
        action = np.zeros(len(env.action_nvec), dtype=np.uint32)
        for _ in range(env.episode_steps):
            env.step(action)
        with pytest.raises(ValueError):
            env.step(action)

    def test_reset_after_truncation_works(self):
        """Reset after truncation allows stepping again."""
        env = native.KsimEnv(TINY_SCENARIO)
        env.reset(seed=0)
        action = np.zeros(len(env.action_nvec), dtype=np.uint32)
        for _ in range(env.episode_steps):
            env.step(action)
        env.reset(seed=1)
        obs, _, _, truncated = env.step(action)
        assert isinstance(obs, np.ndarray)
        assert not truncated


class TestKsimEnvScenario:
    """Test scenario loading."""

    def test_bad_scenario_path_raises_error(self):
        """Loading a non-existent scenario raises ValueError or OSError."""
        with pytest.raises((ValueError, OSError)):
            native.KsimEnv("/nonexistent/path/to/scenario.toml")


class TestKsimEnvDeterminism:
    """Test determinism."""

    def test_deterministic_with_same_seed_and_actions(self):
        """Deterministic trajectory with same seed and actions."""
        env1 = native.KsimEnv(TINY_SCENARIO)
        env2 = native.KsimEnv(TINY_SCENARIO)
        obs1 = env1.reset(seed=42)
        obs2 = env2.reset(seed=42)
        np.testing.assert_array_equal(obs1, obs2)
        action = np.ones(len(env1.action_nvec), dtype=np.uint32)
        for _ in range(5):
            obs1, reward1, _, _ = env1.step(action)
            obs2, reward2, _, _ = env2.step(action)
            np.testing.assert_array_equal(obs1, obs2)
            assert reward1 == reward2


class TestKsimEnvReplicasAndNodes:
    """Test control='replicas_and_nodes' option."""

    def test_replicas_and_nodes_extends_action_nvec(self):
        """control='replicas_and_nodes' adds one 3 per pool to action_nvec."""
        env_replicas = native.KsimEnv(TINY_SCENARIO, control="replicas")
        env_both = native.KsimEnv(TINY_SCENARIO, control="replicas_and_nodes")
        nvec_replicas = env_replicas.action_nvec
        nvec_both = env_both.action_nvec
        # The difference should be 1 pool dimension (value 3)
        assert len(nvec_both) == len(nvec_replicas) + 1
        assert nvec_both[-1] == 3


class TestKsimEnvServiceSlots:
    """Test service_slots option."""

    def test_service_slots_pads_observation_len(self):
        """service_slots=4 pads observation to 4 services."""
        env_default = native.KsimEnv(TINY_SCENARIO)
        env_padded = native.KsimEnv(TINY_SCENARIO, service_slots=4)
        obs_default = env_default.observation_len
        obs_padded = env_padded.observation_len
        # Observation: service_slots * 10 + pools * 5 + 2
        # Default: 2 services, env_padded: 4 services
        assert obs_padded == obs_default + (4 - 2) * 10


class TestKsimEnvMetrics:
    """Test episode metrics."""

    def test_episode_metrics_empty_before_step(self):
        """episode_metrics returns empty dict before any step."""
        env = native.KsimEnv(TINY_SCENARIO)
        env.reset(seed=0)
        metrics = env.episode_metrics()
        assert isinstance(metrics, dict)
        # Before first step, metrics should be empty or minimal
        assert metrics == {}

    def test_episode_metrics_non_empty_after_steps(self):
        """episode_metrics returns non-empty dict after some steps."""
        env = native.KsimEnv(TINY_SCENARIO)
        env.reset(seed=0)
        action = np.zeros(len(env.action_nvec), dtype=np.uint32)
        env.step(action)
        metrics = env.episode_metrics()
        assert isinstance(metrics, dict)
        assert len(metrics) > 0
        assert all(isinstance(k, str) and isinstance(v, float) for k, v in metrics.items())

    def test_episode_metrics_after_full_episode(self):
        """episode_metrics returns metrics after full episode."""
        env = native.KsimEnv(TINY_SCENARIO)
        env.reset(seed=0)
        action = np.zeros(len(env.action_nvec), dtype=np.uint32)
        for _ in range(env.episode_steps):
            env.step(action)
        metrics = env.episode_metrics()
        assert isinstance(metrics, dict)
        assert len(metrics) > 0


class TestKsimVectorEnvProperties:
    """Test basic properties of KsimVectorEnv."""

    def test_num_envs(self):
        """num_envs returns the number of parallel environments."""
        env = native.KsimVectorEnv(TINY_SCENARIO, num_envs=4)
        assert env.num_envs == 4

    def test_observation_len(self):
        """observation_len matches single env."""
        single_env = native.KsimEnv(TINY_SCENARIO)
        vec_env = native.KsimVectorEnv(TINY_SCENARIO, num_envs=2)
        assert vec_env.observation_len == single_env.observation_len

    def test_action_nvec(self):
        """action_nvec matches single env."""
        single_env = native.KsimEnv(TINY_SCENARIO)
        vec_env = native.KsimVectorEnv(TINY_SCENARIO, num_envs=2)
        assert vec_env.action_nvec == single_env.action_nvec


class TestKsimVectorEnvReset:
    """Test KsimVectorEnv reset."""

    def test_reset_shape(self):
        """reset returns shape (num_envs, observation_len)."""
        num_envs = 4
        env = native.KsimVectorEnv(TINY_SCENARIO, num_envs=num_envs)
        obs = env.reset(seed=0)
        assert isinstance(obs, np.ndarray)
        assert obs.dtype == np.float32
        assert obs.shape == (num_envs, env.observation_len)

    def test_reset_observations_within_bounds(self):
        """reset returns all observations within bounds."""
        env = native.KsimVectorEnv(TINY_SCENARIO, num_envs=4)
        obs = env.reset(seed=0)
        assert np.all(obs >= native.OBSERVATION_LOW)
        assert np.all(obs <= native.OBSERVATION_HIGH)

    def test_vector_env_seed_determinism(self):
        """Vector env i uses seed seed+i for same initial obs."""
        num_envs = 3
        vec_env = native.KsimVectorEnv(TINY_SCENARIO, num_envs=num_envs)
        vec_obs = vec_env.reset(seed=100)
        for i in range(num_envs):
            single_env = native.KsimEnv(TINY_SCENARIO)
            single_obs = single_env.reset(seed=100 + i)
            np.testing.assert_array_equal(vec_obs[i], single_obs)


class TestKsimVectorEnvStep:
    """Test KsimVectorEnv step."""

    def test_step_shapes_and_dtypes(self):
        """step returns (obs, rewards, terminated, truncated) with correct shapes and dtypes."""
        num_envs = 4
        env = native.KsimVectorEnv(TINY_SCENARIO, num_envs=num_envs)
        env.reset(seed=0)
        actions = np.zeros((num_envs, len(env.action_nvec)), dtype=np.uint32)
        obs, rewards, terminated, truncated = env.step(actions)
        assert obs.shape == (num_envs, env.observation_len)
        assert obs.dtype == np.float32
        assert rewards.shape == (num_envs,)
        assert rewards.dtype == np.float32
        assert terminated.shape == (num_envs,)
        assert terminated.dtype == np.bool_
        assert truncated.shape == (num_envs,)
        assert truncated.dtype == np.bool_

    def test_step_observations_within_bounds(self):
        """step returns all observations within bounds."""
        env = native.KsimVectorEnv(TINY_SCENARIO, num_envs=4)
        env.reset(seed=0)
        actions = np.zeros((4, len(env.action_nvec)), dtype=np.uint32)
        obs, _, _, _ = env.step(actions)
        assert np.all(obs >= native.OBSERVATION_LOW)
        assert np.all(obs <= native.OBSERVATION_HIGH)

    def test_vector_env_autoreset_behavior(self):
        """Vector env autoresets truncated env on next step with NEXT_STEP mode."""
        env = native.KsimVectorEnv(TINY_SCENARIO, num_envs=2)
        env.reset(seed=0)
        actions = np.zeros((2, len(env.action_nvec)), dtype=np.uint32)
        for step_num in range(env.first().episode_steps + 2):
            _, rewards, terminated, truncated = env.step(actions)
            if step_num >= env.first().episode_steps:
                # After truncation, next step should reset
                assert np.all(rewards == 0.0)
                assert np.all(~terminated)
                assert np.all(~truncated)
