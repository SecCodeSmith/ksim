//! C view of `ksim_env::Env`. Functions returning `i32` return `KSIM_OK`, `KSIM_ERROR` (the
//! message is available from `ksim_last_error` on the same thread) or `KSIM_NULL_POINTER`.
//! Buffers are owned by the caller; lengths are element counts.

#![allow(
    unsafe_code,
    reason = "a C ABI takes raw pointers from the caller and exports unmangled symbols"
)]

use std::ffi::{c_char, c_double, c_float};

pub const KSIM_OK: i32 = 0;
pub const KSIM_ERROR: i32 = -1;
pub const KSIM_NULL_POINTER: i32 = -2;

/// Opaque handle to one environment.
#[derive(Debug)]
pub struct KsimEnv {
    _private: (),
}

/// The last error message on this thread as a NUL-terminated string owned by the library,
/// valid until the next failing call on the thread; NULL when no call has failed.
#[unsafe(no_mangle)]
pub extern "C" fn ksim_last_error() -> *const c_char {
    todo!()
}

/// Loads the scenario TOML at `scenario_path` with default `EnvConfig` except
/// `max_replica_step`. Returns NULL on error (see `ksim_last_error`).
///
/// # Safety
/// `scenario_path` is NULL or a valid NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ksim_env_new(
    scenario_path: *const c_char,
    max_replica_step: u32,
) -> *mut KsimEnv {
    let _ = (scenario_path, max_replica_step);
    todo!()
}

/// Frees an environment; NULL is ignored.
///
/// # Safety
/// `env` is NULL or a pointer returned by `ksim_env_new` that was not freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ksim_env_free(env: *mut KsimEnv) {
    let _ = env;
    todo!()
}

/// Observation length, or 0 for NULL.
///
/// # Safety
/// `env` is NULL or a live environment.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ksim_env_observation_len(env: *const KsimEnv) -> usize {
    let _ = env;
    todo!()
}

/// Number of `MultiDiscrete` action entries, or 0 for NULL.
///
/// # Safety
/// `env` is NULL or a live environment.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ksim_env_action_len(env: *const KsimEnv) -> usize {
    let _ = env;
    todo!()
}

/// Writes the action sizes (`action_nvec`) into `out` of length `len`, which must equal
/// `ksim_env_action_len`.
///
/// # Safety
/// `env` is NULL or live; `out` is NULL or valid for `len` writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ksim_env_action_nvec(
    env: *const KsimEnv,
    out: *mut u32,
    len: usize,
) -> i32 {
    let _ = (env, out, len);
    todo!()
}

/// `Env::reset` into `observation` of length `len`.
///
/// # Safety
/// `env` is NULL or live; `observation` is NULL or valid for `len` writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ksim_env_reset(
    env: *mut KsimEnv,
    seed: u64,
    observation: *mut c_float,
    len: usize,
) -> i32 {
    let _ = (env, seed, observation, len);
    todo!()
}

/// `Env::step`; writes the observation, the reward and the two flags (0 or 1).
///
/// # Safety
/// `env` is NULL or live; every pointer is NULL or valid for its length (1 for the scalars).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ksim_env_step(
    env: *mut KsimEnv,
    action: *const u32,
    action_len: usize,
    observation: *mut c_float,
    observation_len: usize,
    reward: *mut c_double,
    terminated: *mut u8,
    truncated: *mut u8,
) -> i32 {
    let _ = (
        env,
        action,
        action_len,
        observation,
        observation_len,
        reward,
        terminated,
        truncated,
    );
    todo!()
}
