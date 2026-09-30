mod common;

use ksim_core::{
    observation::{Observation, PoolObservation, ServiceObservation},
    performance::ServiceSignals,
    resources::Resources,
    time::SimTime,
};
use ksim_env::{
    EnvError,
    actions::{decode, validate},
    layout::{Control, EnvConfig, Layout},
};

#[test]
fn validate_correct_length() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let action = vec![3; layout.action_nvec().len()];
    let result = validate(&action, &layout);
    assert!(result.is_ok());
}

#[test]
fn validate_wrong_length_too_short() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let action = vec![3; layout.action_nvec().len() - 1];
    let result = validate(&action, &layout);

    assert!(matches!(result, Err(EnvError::BufferLength { .. })));
}

#[test]
fn validate_wrong_length_too_long() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let action = vec![3; layout.action_nvec().len() + 1];
    let result = validate(&action, &layout);

    assert!(matches!(result, Err(EnvError::BufferLength { .. })));
}

#[test]
fn validate_action_out_of_range() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let mut action = vec![3; layout.action_nvec().len()];
    let bound = layout.action_nvec()[0];
    action[0] = bound;

    let result = validate(&action, &layout);
    assert!(matches!(result, Err(EnvError::ActionOutOfRange { .. })));
}

#[test]
fn decode_delta_semantics() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        max_replica_step: 3,
        control: Control::Replicas,
        ..Default::default()
    };
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation {
            desired: 10,
            ready: 10,
            pending: 0,
            unschedulable: 0,
            outdated: 0,
            requests: Resources {
                cpu_millis: 500,
                memory_bytes: 512 * 1024 * 1024,
            },
            limits: Resources {
                cpu_millis: 1000,
                memory_bytes: 1024 * 1024 * 1024,
            },
            signals: ServiceSignals::default(),
        }],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let action = vec![5, 3];
    let actions = decode(&action, &obs, &layout);

    assert_eq!(actions.replicas.len(), 1);
    assert_eq!(actions.replicas[0], Some(12));
}

#[test]
fn decode_clamping_to_min() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        max_replica_step: 3,
        control: Control::Replicas,
        ..Default::default()
    };
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation {
            desired: 5,
            ready: 5,
            pending: 0,
            unschedulable: 0,
            outdated: 0,
            requests: Resources {
                cpu_millis: 500,
                memory_bytes: 512 * 1024 * 1024,
            },
            limits: Resources {
                cpu_millis: 1000,
                memory_bytes: 1024 * 1024 * 1024,
            },
            signals: ServiceSignals::default(),
        }],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let action = vec![0, 3];
    let actions = decode(&action, &obs, &layout);

    assert_eq!(actions.replicas[0], Some(2));
}

#[test]
fn decode_clamping_to_max() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        max_replica_step: 3,
        control: Control::Replicas,
        ..Default::default()
    };
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation {
            desired: 18,
            ready: 18,
            pending: 0,
            unschedulable: 0,
            outdated: 0,
            requests: Resources {
                cpu_millis: 500,
                memory_bytes: 512 * 1024 * 1024,
            },
            limits: Resources {
                cpu_millis: 1000,
                memory_bytes: 1024 * 1024 * 1024,
            },
            signals: ServiceSignals::default(),
        }],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let action = vec![6, 3];
    let actions = decode(&action, &obs, &layout);

    assert_eq!(actions.replicas[0], Some(20));
}

#[test]
fn decode_padding_slots_ignored() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        max_replica_step: 3,
        control: Control::Replicas,
        service_slots: Some(4),
        ..Default::default()
    };
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![
            ServiceObservation {
                desired: 5,
                ready: 5,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources {
                    cpu_millis: 500,
                    memory_bytes: 512 * 1024 * 1024,
                },
                limits: Resources {
                    cpu_millis: 1000,
                    memory_bytes: 1024 * 1024 * 1024,
                },
                signals: ServiceSignals::default(),
            },
            ServiceObservation {
                desired: 3,
                ready: 3,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources {
                    cpu_millis: 1000,
                    memory_bytes: 1024 * 1024 * 1024,
                },
                limits: Resources {
                    cpu_millis: 2000,
                    memory_bytes: 2048 * 1024 * 1024,
                },
                signals: ServiceSignals::default(),
            },
        ],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let action = vec![3, 3, 99, 88, 3];
    let actions = decode(&action, &obs, &layout);

    assert_eq!(actions.replicas.len(), 2);
    assert_eq!(actions.replicas[0], Some(5));
    assert_eq!(actions.replicas[1], Some(3));
}

#[test]
fn decode_pool_delta_with_replicas_and_nodes() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        max_replica_step: 3,
        control: Control::ReplicasAndNodes,
        ..Default::default()
    };
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation::default()],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let action = vec![3, 2];
    let actions = decode(&action, &obs, &layout);

    assert_eq!(actions.pool_delta.len(), 1);
    assert_eq!(actions.pool_delta[0], 1);
}

#[test]
fn decode_no_pool_delta_with_replicas_only() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        max_replica_step: 3,
        control: Control::Replicas,
        ..Default::default()
    };
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation::default()],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let action = vec![3, 3];
    let actions = decode(&action, &obs, &layout);

    assert!(actions.pool_delta.is_empty());
}

#[test]
fn decode_multiple_services_and_pools() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        max_replica_step: 2,
        control: Control::ReplicasAndNodes,
        ..Default::default()
    };
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![
            ServiceObservation {
                desired: 5,
                ..Default::default()
            },
            ServiceObservation {
                desired: 3,
                ..Default::default()
            },
        ],
        pools: vec![PoolObservation::default(), PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let action = vec![2, 2, 1, 2];
    let actions = decode(&action, &obs, &layout);

    assert_eq!(actions.replicas.len(), 2);
    assert_eq!(actions.replicas[0], Some(5));
    assert_eq!(actions.replicas[1], Some(3));
    assert_eq!(actions.pool_delta.len(), 2);
    assert_eq!(actions.pool_delta[0], 0);
    assert_eq!(actions.pool_delta[1], 1);
}
