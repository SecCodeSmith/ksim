"""Tests for the Gymnasium wrappers (KsimGymEnv, KsimVectorGymEnv)."""

from pathlib import Path

import gymnasium
import ksim._native as native
import numpy as np
import pytest
from ksim.gym import EnvOptions, KsimGymEnv, KsimVectorGymEnv, make, make_vec

FIXTURES_DIR = Path(__file__).parent / "fixtures"
TINY_SCENARIO = str(FIXTURES_DIR / "scenarios" / "tiny.toml")


class TestMakeFunction:
    """Test the make factory function."""

    def test_make_creates_env(self):
        """make(scenario) creates a KsimGymEnv."""
        env = make(TINY_SCENARIO)
        assert isinstance(env, KsimGymEnv)

    def test_make_with_string_path(self):
        """make works with string path."""
        env = make(TINY_SCENARIO)
        assert isinstance(env, KsimGymEnv)

    def test_make_with_path_object(self):
        """make works with Path object."""
        env = make(Path(TINY_SCENARIO))
        assert isinstance(env, KsimGymEnv)

    def test_make_with_options(self):
        """make accepts keyword arguments for options."""
        env = make(TINY_SCENARIO, max_replica_step=1)
        assert isinstance(env, KsimGymEnv)


class TestKsimGymEnvProperties:
    """Test KsimGymEnv observation and action spaces."""

    def test_observation_space_is_box(self):
        """observation_space is a Box."""
        env = make(TINY_SCENARIO)
        assert isinstance(env.observation_space, gymnasium.spaces.Box)

    def test_observation_space_bounds(self):
        """observation_space has correct bounds."""
        env = make(TINY_SCENARIO)
        space = env.observation_space
        assert space.low.dtype == np.float32
        assert space.high.dtype == np.float32
        assert np.all(space.low == native.OBSERVATION_LOW)
        assert np.all(space.high == native.OBSERVATION_HIGH)

    def test_observation_space_shape(self):
        """observation_space shape matches observation_len."""
        env = make(TINY_SCENARIO)
        native_env = native.KsimEnv(TINY_SCENARIO)
        assert env.observation_space.shape == (native_env.observation_len,)

    def test_action_space_is_multi_discrete(self):
        """action_space is MultiDiscrete."""
        env = make(TINY_SCENARIO)
        assert isinstance(env.action_space, gymnasium.spaces.MultiDiscrete)

    def test_action_space_nvec(self):
        """action_space.nvec matches native action_nvec."""
        env = make(TINY_SCENARIO)
        native_env = native.KsimEnv(TINY_SCENARIO)
        np.testing.assert_array_equal(env.action_space.nvec, native_env.action_nvec)

    def test_metadata_render_modes_empty(self):
        """metadata['render_modes'] is empty."""
        env = make(TINY_SCENARIO)
        assert env.metadata.get("render_modes", []) == []


class TestKsimGymEnvReset:
    """Test KsimGymEnv reset."""

    def test_reset_returns_tuple(self):
        """reset returns (observation, info)."""
        env = make(TINY_SCENARIO)
        result = env.reset(seed=0)
        assert isinstance(result, tuple)
        assert len(result) == 2

    def test_reset_returns_observation_and_dict(self):
        """reset returns (ndarray, dict)."""
        env = make(TINY_SCENARIO)
        obs, info = env.reset(seed=0)
        assert isinstance(obs, np.ndarray)
        assert isinstance(info, dict)

    def test_reset_observation_shape_and_dtype(self):
        """reset observation has correct shape and dtype."""
        env = make(TINY_SCENARIO)
        obs, _ = env.reset(seed=0)
        assert obs.dtype == np.float32
        assert obs.shape == (env.observation_space.shape[0],)

    def test_reset_with_seed_deterministic(self):
        """reset(seed=s) twice produces same observation."""
        env = make(TINY_SCENARIO)
        obs1, _ = env.reset(seed=42)
        obs2, _ = env.reset(seed=42)
        np.testing.assert_array_equal(obs1, obs2)

    def test_reset_without_seed_continues_sequence(self):
        """reset() without seed continues the PRNG sequence."""
        env = make(TINY_SCENARIO)
        env.reset(seed=100)
        obs1, _ = env.reset()
        obs2, _ = env.reset()
        assert not np.allclose(obs1, obs2)

    def test_reset_info_dict_empty(self):
        """reset info dict is empty."""
        env = make(TINY_SCENARIO)
        _, info = env.reset(seed=0)
        assert info == {}


class TestKsimGymEnvStep:
    """Test KsimGymEnv step."""

    def test_step_returns_tuple_of_five(self):
        """step returns (obs, reward, terminated, truncated, info)."""
        env = make(TINY_SCENARIO)
        env.reset(seed=0)
        action = env.action_space.sample()
        result = env.step(action)
        assert isinstance(result, tuple)
        assert len(result) == 5

    def test_step_with_int64_action(self):
        """step accepts int64 action from action_space.sample()."""
        env = make(TINY_SCENARIO)
        env.reset(seed=0)
        action = env.action_space.sample()
        assert action.dtype == np.int64
        obs, reward, terminated, truncated, info = env.step(action)
        assert isinstance(obs, np.ndarray)
        assert obs.dtype == np.float32
        assert isinstance(reward, (float, np.floating))
        assert isinstance(terminated, (bool, np.bool_))
        assert isinstance(truncated, (bool, np.bool_))
        assert isinstance(info, dict)

    def test_step_returns_correct_types(self):
        """step returns correct types."""
        env = make(TINY_SCENARIO)
        env.reset(seed=0)
        action = np.zeros(len(env.action_space.nvec), dtype=np.int64)
        obs, reward, terminated, truncated, info = env.step(action)
        assert isinstance(obs, np.ndarray)
        assert obs.dtype == np.float32
        assert isinstance(reward, (float, np.floating))
        assert isinstance(terminated, (bool, np.bool_))
        assert isinstance(truncated, (bool, np.bool_))
        assert isinstance(info, dict)

    def test_step_observation_in_space(self):
        """step observation is in observation_space."""
        env = make(TINY_SCENARIO)
        env.reset(seed=0)
        action = np.zeros(len(env.action_space.nvec), dtype=np.int64)
        obs, _, _, _, _ = env.step(action)
        assert env.observation_space.contains(obs)

    def test_step_info_empty_before_truncation(self):
        """step info dict is empty before truncation."""
        env = make(TINY_SCENARIO)
        env.reset(seed=0)
        action = np.zeros(len(env.action_space.nvec), dtype=np.int64)
        for _ in range(env.unwrapped.episode_steps - 1):
            _, _, _, truncated, info = env.step(action)
            assert not truncated
            assert info == {}

    def test_step_info_contains_episode_metrics_on_truncation(self):
        """step info contains 'episode_metrics' on truncation."""
        env = make(TINY_SCENARIO)
        env.reset(seed=0)
        action = np.zeros(len(env.action_space.nvec), dtype=np.int64)
        for _ in range(env.unwrapped.episode_steps - 1):
            env.step(action)
        _, _, _, truncated, info = env.step(action)
        assert truncated
        assert "episode_metrics" in info
        assert isinstance(info["episode_metrics"], dict)
        assert len(info["episode_metrics"]) > 0

    def test_step_truncates_after_episode_steps(self):
        """step returns truncated=True after episode_steps."""
        env = make(TINY_SCENARIO)
        env.reset(seed=0)
        action = np.zeros(len(env.action_space.nvec), dtype=np.int64)
        for _ in range(env.unwrapped.episode_steps - 1):
            _, _, _, truncated, _ = env.step(action)
            assert not truncated
        _, _, _, truncated, _ = env.step(action)
        assert truncated


class TestKsimGymEnvOptions:
    """Test EnvOptions and option handling."""

    def test_env_options_defaults(self):
        """EnvOptions has correct default values."""
        opts = EnvOptions()
        assert opts.max_replica_step == 3
        assert opts.control == "replicas"
        assert opts.service_slots is None
        assert opts.reward_weights == (1.0, 0.5, 0.01)

    def test_env_options_frozen(self):
        """EnvOptions is frozen (immutable)."""
        opts = EnvOptions()
        with pytest.raises(AttributeError):
            opts.max_replica_step = 5

    def test_make_with_max_replica_step(self):
        """make with max_replica_step=1 gives action_nvec entries 3."""
        env1 = make(TINY_SCENARIO, max_replica_step=1)
        env2 = make(TINY_SCENARIO, max_replica_step=3)
        nvec1 = env1.action_space.nvec
        nvec2 = env2.action_space.nvec
        # With max_replica_step=1, action range is [0, 3] (2*1+1=3)
        # With max_replica_step=3, action range is [0, 7] (2*3+1=7)
        assert nvec1[0] == 3
        assert nvec2[0] == 7

    def test_make_with_control_replicas_and_nodes(self):
        """make with control='replicas_and_nodes' extends action_nvec."""
        env = make(TINY_SCENARIO, control="replicas_and_nodes")
        nvec = env.action_space.nvec
        native_env = native.KsimEnv(TINY_SCENARIO, control="replicas_and_nodes")
        np.testing.assert_array_equal(nvec, native_env.action_nvec)

    def test_make_with_service_slots(self):
        """make with service_slots pads observation space."""
        env1 = make(TINY_SCENARIO, service_slots=2)
        env2 = make(TINY_SCENARIO, service_slots=4)
        obs_len1 = env1.observation_space.shape[0]
        obs_len2 = env2.observation_space.shape[0]
        # 4 services pad vs 2 services = +2*10 dimensions
        assert obs_len2 == obs_len1 + (4 - 2) * 10


class TestKsimGymEnvChecker:
    """Test Gymnasium environment checker."""

    def test_check_env_passes(self):
        """gymnasium.utils.env_checker.check_env passes."""
        env = make(TINY_SCENARIO)
        try:
            gymnasium.utils.env_checker.check_env(env.unwrapped, skip_render_check=True)
        except AssertionError as e:
            pytest.fail(f"check_env failed: {e}")


class TestMakeVecFunction:
    """Test the make_vec factory function."""

    def test_make_vec_creates_vector_env(self):
        """make_vec creates a KsimVectorGymEnv."""
        env = make_vec(TINY_SCENARIO, num_envs=4)
        assert isinstance(env, KsimVectorGymEnv)

    def test_make_vec_with_string_path(self):
        """make_vec works with string path."""
        env = make_vec(TINY_SCENARIO, num_envs=4)
        assert isinstance(env, KsimVectorGymEnv)

    def test_make_vec_with_path_object(self):
        """make_vec works with Path object."""
        env = make_vec(Path(TINY_SCENARIO), num_envs=4)
        assert isinstance(env, KsimVectorGymEnv)


class TestKsimVectorGymEnvProperties:
    """Test KsimVectorGymEnv properties."""

    def test_num_envs(self):
        """num_envs matches constructor argument."""
        env = make_vec(TINY_SCENARIO, num_envs=4)
        assert env.num_envs == 4

    def test_single_observation_space_matches_single_env(self):
        """single_observation_space equals the single env's."""
        single_env = make(TINY_SCENARIO)
        vec_env = make_vec(TINY_SCENARIO, num_envs=4)
        assert vec_env.single_observation_space == single_env.observation_space

    def test_single_action_space_matches_single_env(self):
        """single_action_space equals the single env's."""
        single_env = make(TINY_SCENARIO)
        vec_env = make_vec(TINY_SCENARIO, num_envs=4)
        assert vec_env.single_action_space == single_env.action_space

    def test_observation_space_shape(self):
        """observation_space shape is (num_envs, obs_len)."""
        num_envs = 4
        env = make_vec(TINY_SCENARIO, num_envs=num_envs)
        single_env = make(TINY_SCENARIO)
        obs_len = single_env.observation_space.shape[0]
        assert env.observation_space.shape == (num_envs, obs_len)

    def test_action_space_shape(self):
        """action_space shape is (num_envs, action_size)."""
        num_envs = 4
        env = make_vec(TINY_SCENARIO, num_envs=num_envs)
        single_env = make(TINY_SCENARIO)
        action_size = len(single_env.action_space.nvec)
        assert env.action_space.shape == (num_envs, action_size)

    def test_metadata_autoreset_mode(self):
        """metadata['autoreset_mode'] is AutoresetMode.NEXT_STEP."""
        env = make_vec(TINY_SCENARIO, num_envs=4)
        assert env.metadata["autoreset_mode"] == gymnasium.vector.AutoresetMode.NEXT_STEP


class TestKsimVectorGymEnvReset:
    """Test KsimVectorGymEnv reset."""

    def test_reset_returns_tuple(self):
        """reset returns (observations, info)."""
        env = make_vec(TINY_SCENARIO, num_envs=4)
        result = env.reset(seed=0)
        assert isinstance(result, tuple)
        assert len(result) == 2

    def test_reset_observation_shape_dtype(self):
        """reset observations have shape (num_envs, obs_len) and dtype float32."""
        num_envs = 4
        env = make_vec(TINY_SCENARIO, num_envs=num_envs)
        obs, _ = env.reset(seed=0)
        single_env = make(TINY_SCENARIO)
        obs_len = single_env.observation_space.shape[0]
        assert obs.shape == (num_envs, obs_len)
        assert obs.dtype == np.float32

    def test_reset_info_dict_empty(self):
        """reset info dict is empty."""
        env = make_vec(TINY_SCENARIO, num_envs=4)
        _, info = env.reset(seed=0)
        assert info == {}

    def test_reset_with_seed_deterministic(self):
        """reset(seed=s) is deterministic."""
        env1 = make_vec(TINY_SCENARIO, num_envs=4)
        env2 = make_vec(TINY_SCENARIO, num_envs=4)
        obs1, _ = env1.reset(seed=42)
        obs2, _ = env2.reset(seed=42)
        np.testing.assert_array_equal(obs1, obs2)

    def test_vector_env_individual_seeds(self):
        """Vector env i uses seed seed+i."""
        num_envs = 3
        vec_env = make_vec(TINY_SCENARIO, num_envs=num_envs)
        vec_obs, _ = vec_env.reset(seed=100)
        for i in range(num_envs):
            single_env = make(TINY_SCENARIO)
            single_obs, _ = single_env.reset(seed=100 + i)
            np.testing.assert_array_equal(vec_obs[i], single_obs)


class TestKsimVectorGymEnvStep:
    """Test KsimVectorGymEnv step."""

    def test_step_shapes_dtypes(self):
        """step returns correct shapes and dtypes."""
        num_envs = 4
        env = make_vec(TINY_SCENARIO, num_envs=num_envs)
        env.reset(seed=0)
        actions = env.action_space.sample()
        obs, rewards, terminated, truncated, info = env.step(actions)
        single_env = make(TINY_SCENARIO)
        obs_len = single_env.observation_space.shape[0]
        assert obs.shape == (num_envs, obs_len)
        assert obs.dtype == np.float32
        assert rewards.shape == (num_envs,)
        assert rewards.dtype == np.float32
        assert terminated.shape == (num_envs,)
        assert terminated.dtype == np.bool_
        assert truncated.shape == (num_envs,)
        assert truncated.dtype == np.bool_
        assert isinstance(info, dict)

    def test_step_autoreset_after_truncation(self):
        """After truncation, next step autoresets with NEXT_STEP mode."""
        env = make_vec(TINY_SCENARIO, num_envs=2)
        env.reset(seed=0)
        single_env = make(TINY_SCENARIO)
        episode_steps = single_env.unwrapped.episode_steps
        actions = np.zeros((2, len(single_env.action_space.nvec)), dtype=np.int64)
        for _ in range(episode_steps):
            env.step(actions)
        _, rewards, terminated, truncated, _ = env.step(actions)
        assert np.all(rewards == 0.0)
        assert np.all(~terminated)
        assert np.all(~truncated)

    def test_step_all_envs_truncate_together(self):
        """All vector envs truncate at the same step number."""
        env = make_vec(TINY_SCENARIO, num_envs=4)
        env.reset(seed=0)
        single_env = make(TINY_SCENARIO)
        episode_steps = single_env.unwrapped.episode_steps
        actions = np.zeros((4, len(single_env.action_space.nvec)), dtype=np.int64)
        for step_num in range(episode_steps + 1):
            _, _, _, truncated, _ = env.step(actions)
            if step_num < episode_steps:
                assert not np.any(truncated)
            else:
                assert np.all(truncated)
