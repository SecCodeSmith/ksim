use ksim_perf::queueing::erlang_c;
use proptest::prelude::*;

#[test]
fn erlang_c_m_m_1_half_load() {
    // M/M/1 queue with offered load 0.5: C = rho = 0.5
    let c = erlang_c(1, 0.5);
    approx::assert_abs_diff_eq!(c, 0.5, epsilon = 1e-9);
}

#[test]
fn erlang_c_m_m_2_unity_load() {
    // M/M/2 queue with offered load 1.0: C = 1/3
    let c = erlang_c(2, 1.0);
    approx::assert_abs_diff_eq!(c, 1.0 / 3.0, epsilon = 1e-9);
}

#[test]
fn erlang_c_m_m_2_double_load() {
    // M/M/2 queue with offered load 2.0 (unstable): C = 1
    let c = erlang_c(2, 2.0);
    approx::assert_abs_diff_eq!(c, 1.0, epsilon = 1e-9);
}

#[test]
fn erlang_c_zero_servers() {
    // Zero servers should return 1 (unstable)
    let c = erlang_c(0, 5.0);
    approx::assert_abs_diff_eq!(c, 1.0, epsilon = 1e-9);
}

proptest! {
    #[test]
    fn erlang_c_monotone_increasing_in_load(servers in 1u32..10, load1 in 0.0_f64..2.0, load2 in 0.0_f64..2.0) {
        prop_assume!(load1 <= load2);
        let c1 = erlang_c(servers, load1);
        let c2 = erlang_c(servers, load2);
        prop_assert!(c1 <= c2, "erlang_c must be monotone increasing in offered load: {} <= {}", c1, c2);
    }
}
