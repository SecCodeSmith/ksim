#![allow(unsafe_code, reason = "tests exercise the raw C ABI")]

use std::ffi::CString;

fn scenario_path() -> String {
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../python/tests/fixtures/scenarios/tiny.toml"
    )
    .to_string()
}

#[test]
fn ksim_env_new_with_null_path_returns_null_and_sets_error() {
    unsafe {
        let env = ksim::env::ksim_env_new(std::ptr::null(), 3);
        assert!(
            env.is_null(),
            "ksim_env_new with NULL path should return NULL"
        );
        let error_msg = ksim::env::ksim_last_error();
        assert!(
            !error_msg.is_null(),
            "ksim_last_error should be non-NULL after ksim_env_new fails"
        );
    }
}

#[test]
fn ksim_env_new_with_missing_file_returns_null_with_error() {
    unsafe {
        let missing_path = CString::new("/nonexistent/path.toml").unwrap();
        let env = ksim::env::ksim_env_new(missing_path.as_ptr(), 3);
        assert!(
            env.is_null(),
            "ksim_env_new with missing file should return NULL"
        );
        let error_msg = ksim::env::ksim_last_error();
        assert!(!error_msg.is_null(), "ksim_last_error should be non-NULL");
    }
}

#[test]
fn ksim_env_new_with_valid_scenario_returns_non_null() {
    unsafe {
        let path = CString::new(scenario_path()).unwrap();
        let env = ksim::env::ksim_env_new(path.as_ptr(), 3);
        assert!(
            !env.is_null(),
            "ksim_env_new with valid scenario should return non-NULL"
        );
        ksim::env::ksim_env_free(env);
    }
}

#[test]
fn ksim_env_observation_len_returns_zero_for_null() {
    unsafe {
        let len = ksim::env::ksim_env_observation_len(std::ptr::null());
        assert_eq!(len, 0, "observation_len for NULL should return 0");
    }
}

#[test]
fn ksim_env_observation_len_returns_nonzero_for_valid_env() {
    unsafe {
        let path = CString::new(scenario_path()).unwrap();
        let env = ksim::env::ksim_env_new(path.as_ptr(), 3);
        assert!(!env.is_null());
        let len = ksim::env::ksim_env_observation_len(env);
        assert!(len > 0, "observation_len for valid env should be > 0");
        ksim::env::ksim_env_free(env);
    }
}

#[test]
fn ksim_env_action_len_returns_zero_for_null() {
    unsafe {
        let len = ksim::env::ksim_env_action_len(std::ptr::null());
        assert_eq!(len, 0, "action_len for NULL should return 0");
    }
}

#[test]
fn ksim_env_action_len_returns_nonzero_for_valid_env() {
    unsafe {
        let path = CString::new(scenario_path()).unwrap();
        let env = ksim::env::ksim_env_new(path.as_ptr(), 3);
        assert!(!env.is_null());
        let len = ksim::env::ksim_env_action_len(env);
        assert!(len > 0, "action_len for valid env should be > 0");
        ksim::env::ksim_env_free(env);
    }
}

#[test]
fn ksim_env_action_nvec_with_wrong_length_returns_error() {
    unsafe {
        let path = CString::new(scenario_path()).unwrap();
        let env = ksim::env::ksim_env_new(path.as_ptr(), 3);
        assert!(!env.is_null());
        let action_len = ksim::env::ksim_env_action_len(env);
        let mut out = vec![0u32; action_len + 1];
        let result = ksim::env::ksim_env_action_nvec(env, out.as_mut_ptr(), action_len + 1);
        assert_eq!(
            result,
            ksim::env::KSIM_ERROR,
            "action_nvec with wrong length should return KSIM_ERROR"
        );
        ksim::env::ksim_env_free(env);
    }
}

#[test]
fn ksim_env_action_nvec_with_null_pointer_returns_null_pointer_error() {
    unsafe {
        let path = CString::new(scenario_path()).unwrap();
        let env = ksim::env::ksim_env_new(path.as_ptr(), 3);
        assert!(!env.is_null());
        let action_len = ksim::env::ksim_env_action_len(env);
        let result = ksim::env::ksim_env_action_nvec(env, std::ptr::null_mut(), action_len);
        assert_eq!(
            result,
            ksim::env::KSIM_NULL_POINTER,
            "action_nvec with NULL out should return KSIM_NULL_POINTER"
        );
        ksim::env::ksim_env_free(env);
    }
}

#[test]
fn ksim_env_action_nvec_fills_correct_entries() {
    unsafe {
        let path = CString::new(scenario_path()).unwrap();
        let env = ksim::env::ksim_env_new(path.as_ptr(), 3);
        assert!(!env.is_null());
        let action_len = ksim::env::ksim_env_action_len(env);
        assert_eq!(action_len, 2);
        let mut out = vec![0u32; action_len];
        let result = ksim::env::ksim_env_action_nvec(env, out.as_mut_ptr(), action_len);
        assert_eq!(result, ksim::env::KSIM_OK);
        assert_eq!(out, vec![7, 7]);
        ksim::env::ksim_env_free(env);
    }
}

#[test]
fn ksim_env_reset_returns_ok_and_writes_finite_values() {
    unsafe {
        let path = CString::new(scenario_path()).unwrap();
        let env = ksim::env::ksim_env_new(path.as_ptr(), 3);
        assert!(!env.is_null());
        let obs_len = ksim::env::ksim_env_observation_len(env.cast_const());
        let mut observation = vec![0.0f32; obs_len];
        let result = ksim::env::ksim_env_reset(env, 1, observation.as_mut_ptr(), obs_len);
        assert_eq!(result, ksim::env::KSIM_OK, "reset should return KSIM_OK");
        assert!(
            observation.iter().all(|&x| x.is_finite()),
            "reset should write finite observation values"
        );
        ksim::env::ksim_env_free(env);
    }
}

#[test]
fn ksim_env_step_before_reset_returns_error() {
    unsafe {
        let path = CString::new(scenario_path()).unwrap();
        let env = ksim::env::ksim_env_new(path.as_ptr(), 3);
        assert!(!env.is_null());
        let action_len = ksim::env::ksim_env_action_len(env.cast_const());
        let obs_len = ksim::env::ksim_env_observation_len(env.cast_const());
        let action = vec![3u32; action_len];
        let mut observation = vec![0.0f32; obs_len];
        let mut reward = 0.0f64;
        let mut terminated = 0u8;
        let mut truncated = 0u8;
        let result = ksim::env::ksim_env_step(
            env,
            action.as_ptr(),
            action_len,
            observation.as_mut_ptr(),
            obs_len,
            &raw mut reward,
            &raw mut terminated,
            &raw mut truncated,
        );
        assert_eq!(
            result,
            ksim::env::KSIM_ERROR,
            "step before reset should return KSIM_ERROR"
        );
        ksim::env::ksim_env_free(env);
    }
}

#[test]
fn ksim_env_step_with_zero_delta_action_returns_ok_with_finite_reward() {
    unsafe {
        let path = CString::new(scenario_path()).unwrap();
        let env = ksim::env::ksim_env_new(path.as_ptr(), 3);
        assert!(!env.is_null());
        let action_len = ksim::env::ksim_env_action_len(env.cast_const());
        let obs_len = ksim::env::ksim_env_observation_len(env.cast_const());
        let mut observation = vec![0.0f32; obs_len];
        let _ = ksim::env::ksim_env_reset(env, 1, observation.as_mut_ptr(), obs_len);

        // Zero-delta action means all entries are k (the midpoint, no change)
        let action = vec![3u32; action_len]; // k=3 means no change
        let mut reward = 0.0f64;
        let mut terminated = 0u8;
        let mut truncated = 0u8;
        let result = ksim::env::ksim_env_step(
            env,
            action.as_ptr(),
            action_len,
            observation.as_mut_ptr(),
            obs_len,
            &raw mut reward,
            &raw mut terminated,
            &raw mut truncated,
        );
        assert_eq!(
            result,
            ksim::env::KSIM_OK,
            "step with zero-delta action should return KSIM_OK"
        );
        assert!(reward.is_finite(), "reward should be finite");
        assert!(
            terminated == 0 || terminated == 1,
            "terminated should be 0 or 1"
        );
        assert!(
            truncated == 0 || truncated == 1,
            "truncated should be 0 or 1"
        );
        ksim::env::ksim_env_free(env);
    }
}

#[test]
fn ksim_env_step_with_wrong_observation_buffer_length_returns_error() {
    unsafe {
        let path = CString::new(scenario_path()).unwrap();
        let env = ksim::env::ksim_env_new(path.as_ptr(), 3);
        assert!(!env.is_null());
        let action_len = ksim::env::ksim_env_action_len(env.cast_const());
        let obs_len = ksim::env::ksim_env_observation_len(env.cast_const());
        let mut observation = vec![0.0f32; obs_len];
        let _ = ksim::env::ksim_env_reset(env, 1, observation.as_mut_ptr(), obs_len);

        let action = vec![3u32; action_len];
        let mut wrong_observation = vec![0.0f32; obs_len + 1];
        let mut reward = 0.0f64;
        let mut terminated = 0u8;
        let mut truncated = 0u8;
        let result = ksim::env::ksim_env_step(
            env,
            action.as_ptr(),
            action_len,
            wrong_observation.as_mut_ptr(),
            obs_len + 1,
            &raw mut reward,
            &raw mut terminated,
            &raw mut truncated,
        );
        assert_eq!(
            result,
            ksim::env::KSIM_ERROR,
            "step with wrong observation buffer length should return KSIM_ERROR"
        );
        ksim::env::ksim_env_free(env);
    }
}

#[test]
fn ksim_env_step_with_null_pointers_returns_null_pointer_error() {
    unsafe {
        let path = CString::new(scenario_path()).unwrap();
        let env = ksim::env::ksim_env_new(path.as_ptr(), 3);
        assert!(!env.is_null());
        let action_len = ksim::env::ksim_env_action_len(env.cast_const());
        let obs_len = ksim::env::ksim_env_observation_len(env.cast_const());
        let mut observation = vec![0.0f32; obs_len];
        let _ = ksim::env::ksim_env_reset(env, 1, observation.as_mut_ptr(), obs_len);

        let action = vec![3u32; action_len];
        let mut reward = 0.0f64;
        let mut terminated = 0u8;
        let mut truncated = 0u8;

        let result = ksim::env::ksim_env_step(
            env,
            std::ptr::null(),
            action_len,
            observation.as_mut_ptr(),
            obs_len,
            &raw mut reward,
            &raw mut terminated,
            &raw mut truncated,
        );
        assert_eq!(
            result,
            ksim::env::KSIM_NULL_POINTER,
            "step with NULL action should return KSIM_NULL_POINTER"
        );

        let result = ksim::env::ksim_env_step(
            env,
            action.as_ptr(),
            action_len,
            std::ptr::null_mut(),
            obs_len,
            &raw mut reward,
            &raw mut terminated,
            &raw mut truncated,
        );
        assert_eq!(
            result,
            ksim::env::KSIM_NULL_POINTER,
            "step with NULL observation should return KSIM_NULL_POINTER"
        );

        ksim::env::ksim_env_free(env);
    }
}

#[test]
fn ksim_env_truncated_becomes_one_exactly_on_episode_steps() {
    unsafe {
        let path = CString::new(scenario_path()).unwrap();
        let env = ksim::env::ksim_env_new(path.as_ptr(), 3);
        assert!(!env.is_null());
        let action_len = ksim::env::ksim_env_action_len(env.cast_const());
        let obs_len = ksim::env::ksim_env_observation_len(env.cast_const());
        let mut observation = vec![0.0f32; obs_len];
        let _ = ksim::env::ksim_env_reset(env, 1, observation.as_mut_ptr(), obs_len);

        // For tiny.toml: episode_steps = 150 / 15 = 10
        let expected_steps = 10;

        let action = vec![3u32; action_len];
        let mut reward = 0.0f64;
        let mut terminated = 0u8;
        let mut truncated = 0u8;

        for step_num in 0..expected_steps {
            let result = ksim::env::ksim_env_step(
                env,
                action.as_ptr(),
                action_len,
                observation.as_mut_ptr(),
                obs_len,
                &raw mut reward,
                &raw mut terminated,
                &raw mut truncated,
            );
            assert_eq!(result, ksim::env::KSIM_OK, "step {step_num} should succeed");
            if step_num < expected_steps - 1 {
                assert_eq!(truncated, 0, "truncated should be 0 before episode_steps");
            }
        }
        assert_eq!(
            truncated, 1,
            "truncated should be 1 exactly at episode_steps"
        );
        ksim::env::ksim_env_free(env);
    }
}

#[test]
fn ksim_env_free_with_null_is_noop() {
    unsafe {
        ksim::env::ksim_env_free(std::ptr::null_mut());
    }
}

#[test]
fn ksim_env_free_valid_env() {
    unsafe {
        let path = CString::new(scenario_path()).unwrap();
        let env = ksim::env::ksim_env_new(path.as_ptr(), 3);
        assert!(!env.is_null());
        ksim::env::ksim_env_free(env);
    }
}

#[test]
fn ksim_env_reset_with_wrong_observation_buffer_length_returns_error() {
    unsafe {
        let path = CString::new(scenario_path()).unwrap();
        let env = ksim::env::ksim_env_new(path.as_ptr(), 3);
        assert!(!env.is_null());
        let obs_len = ksim::env::ksim_env_observation_len(env.cast_const());
        let mut observation = vec![0.0f32; obs_len + 1];
        let result = ksim::env::ksim_env_reset(env, 1, observation.as_mut_ptr(), obs_len + 1);
        assert_eq!(
            result,
            ksim::env::KSIM_ERROR,
            "reset with wrong observation buffer length should return KSIM_ERROR"
        );
        ksim::env::ksim_env_free(env);
    }
}

#[test]
fn ksim_env_reset_with_null_observation_returns_null_pointer_error() {
    unsafe {
        let path = CString::new(scenario_path()).unwrap();
        let env = ksim::env::ksim_env_new(path.as_ptr(), 3);
        assert!(!env.is_null());
        let obs_len = ksim::env::ksim_env_observation_len(env.cast_const());
        let result = ksim::env::ksim_env_reset(env, 1, std::ptr::null_mut(), obs_len);
        assert_eq!(
            result,
            ksim::env::KSIM_NULL_POINTER,
            "reset with NULL observation should return KSIM_NULL_POINTER"
        );
        ksim::env::ksim_env_free(env);
    }
}
