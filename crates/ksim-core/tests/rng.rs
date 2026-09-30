use ksim_core::rng::{Concern, RngStreams};
use rand::Rng;

#[test]
fn same_seed_same_concern_identical_sequence() {
    let streams1 = RngStreams::new(42);
    let streams2 = RngStreams::new(42);

    let mut rng1 = streams1.stream(Concern::Lifecycle);
    let mut rng2 = streams2.stream(Concern::Lifecycle);

    for _ in 0..8 {
        let val1 = rng1.next_u64();
        let val2 = rng2.next_u64();
        assert_eq!(val1, val2);
    }
}

#[test]
fn different_concern_different_sequence() {
    let streams = RngStreams::new(42);

    let mut rng_lifecycle = streams.stream(Concern::Lifecycle);
    let mut rng_workload = streams.stream(Concern::Workload);

    let mut vals_lifecycle = Vec::new();
    let mut vals_workload = Vec::new();

    for _ in 0..8 {
        vals_lifecycle.push(rng_lifecycle.next_u64());
        vals_workload.push(rng_workload.next_u64());
    }

    assert_ne!(vals_lifecycle, vals_workload);
}

#[test]
fn different_seed_different_sequence() {
    let streams1 = RngStreams::new(42);
    let streams2 = RngStreams::new(43);

    let mut rng1 = streams1.stream(Concern::Scheduler);
    let mut rng2 = streams2.stream(Concern::Scheduler);

    let mut vals1 = Vec::new();
    let mut vals2 = Vec::new();

    for _ in 0..8 {
        vals1.push(rng1.next_u64());
        vals2.push(rng2.next_u64());
    }

    assert_ne!(vals1, vals2);
}

#[test]
fn same_seed_lifecycle_vs_workload() {
    let streams = RngStreams::new(100);

    let mut rng1 = streams.stream(Concern::Lifecycle);
    let mut rng2 = streams.stream(Concern::Workload);

    let vals1: Vec<u64> = (0..8).map(|_| rng1.next_u64()).collect();
    let vals2: Vec<u64> = (0..8).map(|_| rng2.next_u64()).collect();

    assert_ne!(vals1, vals2);
}

#[test]
fn same_seed_workload_vs_scheduler() {
    let streams = RngStreams::new(100);

    let mut rng1 = streams.stream(Concern::Workload);
    let mut rng2 = streams.stream(Concern::Scheduler);

    let vals1: Vec<u64> = (0..8).map(|_| rng1.next_u64()).collect();
    let vals2: Vec<u64> = (0..8).map(|_| rng2.next_u64()).collect();

    assert_ne!(vals1, vals2);
}

#[test]
fn same_seed_scheduler_vs_lifecycle() {
    let streams = RngStreams::new(100);

    let mut rng1 = streams.stream(Concern::Scheduler);
    let mut rng2 = streams.stream(Concern::Lifecycle);

    let vals1: Vec<u64> = (0..8).map(|_| rng1.next_u64()).collect();
    let vals2: Vec<u64> = (0..8).map(|_| rng2.next_u64()).collect();

    assert_ne!(vals1, vals2);
}

#[test]
fn different_seeds_lifecycle_different() {
    let streams1 = RngStreams::new(123);
    let streams2 = RngStreams::new(456);

    let mut rng1 = streams1.stream(Concern::Lifecycle);
    let mut rng2 = streams2.stream(Concern::Lifecycle);

    let vals1: Vec<u64> = (0..8).map(|_| rng1.next_u64()).collect();
    let vals2: Vec<u64> = (0..8).map(|_| rng2.next_u64()).collect();

    assert_ne!(vals1, vals2);
}

#[test]
fn different_seeds_workload_different() {
    let streams1 = RngStreams::new(789);
    let streams2 = RngStreams::new(1000);

    let mut rng1 = streams1.stream(Concern::Workload);
    let mut rng2 = streams2.stream(Concern::Workload);

    let vals1: Vec<u64> = (0..8).map(|_| rng1.next_u64()).collect();
    let vals2: Vec<u64> = (0..8).map(|_| rng2.next_u64()).collect();

    assert_ne!(vals1, vals2);
}

#[test]
fn different_seeds_scheduler_different() {
    let streams1 = RngStreams::new(111);
    let streams2 = RngStreams::new(222);

    let mut rng1 = streams1.stream(Concern::Scheduler);
    let mut rng2 = streams2.stream(Concern::Scheduler);

    let vals1: Vec<u64> = (0..8).map(|_| rng1.next_u64()).collect();
    let vals2: Vec<u64> = (0..8).map(|_| rng2.next_u64()).collect();

    assert_ne!(vals1, vals2);
}

#[test]
fn independent_streams_same_seed() {
    // Different concerns shouldn't interfere with each other
    let streams = RngStreams::new(555);

    let mut rng_lifecycle = streams.stream(Concern::Lifecycle);
    let mut rng_workload = streams.stream(Concern::Workload);
    let mut rng_scheduler = streams.stream(Concern::Scheduler);

    let mut vals_lifecycle = Vec::new();
    let mut vals_workload = Vec::new();
    let mut vals_scheduler = Vec::new();

    for _ in 0..8 {
        vals_lifecycle.push(rng_lifecycle.next_u64());
        vals_workload.push(rng_workload.next_u64());
        vals_scheduler.push(rng_scheduler.next_u64());
    }

    // All should be different from each other
    assert_ne!(vals_lifecycle, vals_workload);
    assert_ne!(vals_lifecycle, vals_scheduler);
    assert_ne!(vals_workload, vals_scheduler);
}

#[test]
fn identical_streams_for_same_seed_and_concern() {
    let streams1 = RngStreams::new(777);
    let streams2 = RngStreams::new(777);

    let mut rng1 = streams1.stream(Concern::Workload);
    let mut rng2 = streams2.stream(Concern::Workload);

    for _ in 0..8 {
        assert_eq!(rng1.next_u64(), rng2.next_u64());
    }
}

#[test]
fn zero_seed_lifecycle() {
    let streams1 = RngStreams::new(0);
    let streams2 = RngStreams::new(0);

    let mut rng1 = streams1.stream(Concern::Lifecycle);
    let mut rng2 = streams2.stream(Concern::Lifecycle);

    for _ in 0..8 {
        assert_eq!(rng1.next_u64(), rng2.next_u64());
    }
}

#[test]
fn max_seed_lifecycle() {
    let streams1 = RngStreams::new(u64::MAX);
    let streams2 = RngStreams::new(u64::MAX);

    let mut rng1 = streams1.stream(Concern::Lifecycle);
    let mut rng2 = streams2.stream(Concern::Lifecycle);

    for _ in 0..8 {
        assert_eq!(rng1.next_u64(), rng2.next_u64());
    }
}
