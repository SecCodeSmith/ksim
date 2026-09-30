//! C view of `ksim_env::Env`. Functions returning `i32` return `KSIM_OK`, `KSIM_ERROR` (the
//! message is available from `ksim_last_error` on the same thread) or `KSIM_NULL_POINTER`.
//! Buffers are owned by the caller; lengths are element counts.

#![allow(
    unsafe_code,
    reason = "a C ABI takes raw pointers from the caller and exports unmangled symbols"
)]

use std::cell::RefCell;
use std::ffi::{CStr, CString, c_char, c_double, c_float};
use std::fmt::Display;
use std::slice;

use ksim_bench::scenario::Scenario;
use ksim_env::{Env, EnvConfig};

pub const KSIM_OK: i32 = 0;
pub const KSIM_ERROR: i32 = -1;
pub const KSIM_NULL_POINTER: i32 = -2;

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn record_error(message: impl Display) {
    let sanitized = message.to_string().replace('\0', " ");
    let message = CString::new(sanitized).unwrap_or_default();
    LAST_ERROR.with(|slot| *slot.borrow_mut() = Some(message));
}

fn fail(message: impl Display) -> i32 {
    record_error(message);
    KSIM_ERROR
}

fn null_argument(name: &str) -> i32 {
    record_error(format_args!("{name} is NULL"));
    KSIM_NULL_POINTER
}

/// Opaque handle to one environment.
#[derive(Debug)]
pub struct KsimEnv {
    env: Env,
}

/// The last error message on this thread as a NUL-terminated string owned by the library,
/// valid until the next failing call on the thread; NULL when no call has failed.
#[unsafe(no_mangle)]
pub extern "C" fn ksim_last_error() -> *const c_char {
    // The CString's heap buffer is stable until the slot is overwritten by the next failure.
    LAST_ERROR.with(|slot| {
        slot.borrow()
            .as_ref()
            .map_or(std::ptr::null(), |m| m.as_ptr())
    })
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
    if scenario_path.is_null() {
        null_argument("scenario_path");
        return std::ptr::null_mut();
    }
    // SAFETY: non-NULL, and the caller guarantees a valid NUL-terminated string.
    let path = unsafe { CStr::from_ptr(scenario_path) };
    match build_env(path, max_replica_step) {
        Ok(env) => Box::into_raw(Box::new(KsimEnv { env })),
        Err(message) => {
            record_error(message);
            std::ptr::null_mut()
        }
    }
}

fn build_env(path: &CStr, max_replica_step: u32) -> Result<Env, String> {
    let path = path.to_str().map_err(|e| format!("scenario_path: {e}"))?;
    let scenario = Scenario::from_toml_file(path).map_err(|e| e.to_string())?;
    let config = EnvConfig {
        max_replica_step,
        ..EnvConfig::default()
    };
    Env::new(&scenario, &config).map_err(|e| e.to_string())
}

/// Frees an environment; NULL is ignored.
///
/// # Safety
/// `env` is NULL or a pointer returned by `ksim_env_new` that was not freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ksim_env_free(env: *mut KsimEnv) {
    if !env.is_null() {
        // SAFETY: non-NULL, so it came from `ksim_env_new`'s Box::into_raw and is freed once.
        drop(unsafe { Box::from_raw(env) });
    }
}

/// Observation length, or 0 for NULL.
///
/// # Safety
/// `env` is NULL or a live environment.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ksim_env_observation_len(env: *const KsimEnv) -> usize {
    // SAFETY: the caller guarantees `env` is NULL or live.
    unsafe { env.as_ref() }.map_or(0, |handle| handle.env.layout().observation_len())
}

/// Number of `MultiDiscrete` action entries, or 0 for NULL.
///
/// # Safety
/// `env` is NULL or a live environment.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ksim_env_action_len(env: *const KsimEnv) -> usize {
    // SAFETY: the caller guarantees `env` is NULL or live.
    unsafe { env.as_ref() }.map_or(0, |handle| handle.env.layout().action_len())
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
    // SAFETY: the caller guarantees `env` is NULL or live.
    let Some(handle) = (unsafe { env.as_ref() }) else {
        return null_argument("env");
    };
    // SAFETY: the caller guarantees `out` is NULL or valid for `len` writes.
    let Some(out) = (unsafe { buffer_mut(out, len) }) else {
        return null_argument("out");
    };
    let nvec = handle.env.layout().action_nvec();
    if nvec.len() != out.len() {
        return fail(format_args!(
            "out has length {}, expected {}",
            out.len(),
            nvec.len()
        ));
    }
    out.copy_from_slice(&nvec);
    KSIM_OK
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
    // SAFETY: the caller guarantees `env` is NULL or live and not aliased during the call.
    let Some(handle) = (unsafe { env.as_mut() }) else {
        return null_argument("env");
    };
    // SAFETY: the caller guarantees `observation` is NULL or valid for `len` writes.
    let Some(observation) = (unsafe { buffer_mut(observation, len) }) else {
        return null_argument("observation");
    };
    match handle.env.reset(seed, observation) {
        Ok(()) => KSIM_OK,
        Err(error) => fail(error),
    }
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
    // SAFETY: the caller guarantees `env` is NULL or live and not aliased during the call.
    let Some(handle) = (unsafe { env.as_mut() }) else {
        return null_argument("env");
    };
    // SAFETY: the caller guarantees `action` is NULL or valid for `action_len` reads.
    let Some(action) = (unsafe { buffer(action, action_len) }) else {
        return null_argument("action");
    };
    // SAFETY: the caller guarantees `observation` is NULL or valid for `observation_len` writes.
    let Some(observation) = (unsafe { buffer_mut(observation, observation_len) }) else {
        return null_argument("observation");
    };
    // SAFETY: the caller guarantees each scalar pointer is NULL or valid for one write.
    let (Some(reward), Some(terminated), Some(truncated)) =
        (unsafe { (reward.as_mut(), terminated.as_mut(), truncated.as_mut()) })
    else {
        return null_argument("reward, terminated or truncated");
    };
    match handle.env.step(action, observation) {
        Ok(outcome) => {
            *reward = outcome.reward;
            *terminated = u8::from(outcome.terminated);
            *truncated = u8::from(outcome.truncated);
            KSIM_OK
        }
        Err(error) => fail(error),
    }
}

/// # Safety
/// `ptr` is NULL or valid for `len` reads for the lifetime `'a`.
unsafe fn buffer<'a, T>(ptr: *const T, len: usize) -> Option<&'a [T]> {
    // SAFETY: non-NULL and valid for `len` reads per the caller contract.
    (!ptr.is_null()).then(|| unsafe { slice::from_raw_parts(ptr, len) })
}

/// # Safety
/// `ptr` is NULL or valid for `len` writes for the lifetime `'a`, with no other access.
unsafe fn buffer_mut<'a, T>(ptr: *mut T, len: usize) -> Option<&'a mut [T]> {
    // SAFETY: non-NULL and valid for `len` writes per the caller contract.
    (!ptr.is_null()).then(|| unsafe { slice::from_raw_parts_mut(ptr, len) })
}
