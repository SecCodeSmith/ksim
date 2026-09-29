use ksim_autoscalers::{
    stack::KubernetesStack,
    traits::{NodePlan, ReplicaRecommender, ResourcePlan, ResourceRecommender},
};
use ksim_core::{
    actions::Actions,
    autoscaler::Autoscaler,
    ids::NodeId,
    observation::{Observation, PoolObservation, ServiceObservation},
    resources::Resources,
    time::SimTime,
};

#[test]
fn kubernetes_stack_new_returns_none_actions() {
    let mut stack = KubernetesStack::new();

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![ServiceObservation::default()],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let actions = stack.act(&observation);

    // Actions::none() is equivalent to empty vectors
    assert_eq!(actions, Actions::none());
    assert!(actions.replicas.is_empty());
    assert!(actions.requests.is_empty());
    assert!(actions.pool_delta.is_empty());
    assert!(actions.drain.is_empty());
    assert!(actions.evict.is_empty());
}

// Fake implementations for testing the stack
#[derive(Debug)]
struct FakeReplicaRecommender {
    replicas: Vec<Option<u32>>,
}

impl ReplicaRecommender for FakeReplicaRecommender {
    fn recommend_replicas(&mut self, _observation: &Observation) -> Vec<Option<u32>> {
        self.replicas.clone()
    }
}

#[derive(Debug)]
struct FakeResourceRecommender {
    requests: Vec<Option<Resources>>,
    evict: Vec<u32>,
}

impl ResourceRecommender for FakeResourceRecommender {
    fn recommend_resources(&mut self, _observation: &Observation) -> ResourcePlan {
        ResourcePlan {
            requests: self.requests.clone(),
            evict: self.evict.clone(),
        }
    }
}

#[derive(Debug)]
struct FakeNodeProvisioner {
    pool_delta: Vec<i32>,
    drain: Vec<NodeId>,
}

impl ksim_autoscalers::traits::NodeProvisioner for FakeNodeProvisioner {
    fn provision(&mut self, _observation: &Observation) -> NodePlan {
        NodePlan {
            pool_delta: self.pool_delta.clone(),
            drain: self.drain.clone(),
        }
    }
}

#[test]
fn kubernetes_stack_merges_actions_from_all_components() {
    let replicas = vec![Some(5)];
    let requests = vec![Some(Resources::new(500, 512 * 1024 * 1024))];
    let evict_count = vec![2];
    let pool_delta = vec![1];
    let drain = vec![NodeId(42)];

    let stack = KubernetesStack::new()
        .with_replicas(Box::new(FakeReplicaRecommender {
            replicas: replicas.clone(),
        }))
        .with_resources(Box::new(FakeResourceRecommender {
            requests: requests.clone(),
            evict: evict_count.clone(),
        }))
        .with_nodes(Box::new(FakeNodeProvisioner {
            pool_delta: pool_delta.clone(),
            drain: drain.clone(),
        }));

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![ServiceObservation::default()],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut stack = stack;
    let actions = stack.act(&observation);

    assert_eq!(actions.replicas, replicas);
    assert_eq!(actions.requests, requests);
    assert_eq!(actions.evict, evict_count);
    assert_eq!(actions.pool_delta, pool_delta);
    assert_eq!(actions.drain, drain);
}

#[test]
fn kubernetes_stack_only_replicas() {
    let replicas = vec![Some(3)];

    let stack = KubernetesStack::new().with_replicas(Box::new(FakeReplicaRecommender {
        replicas: replicas.clone(),
    }));

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![ServiceObservation::default()],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut stack = stack;
    let actions = stack.act(&observation);

    assert_eq!(actions.replicas, replicas);
    assert!(actions.requests.is_empty());
    assert!(actions.evict.is_empty());
    assert!(actions.pool_delta.is_empty());
    assert!(actions.drain.is_empty());
}

#[test]
fn kubernetes_stack_only_resources() {
    let requests = vec![Some(Resources::new(1000, 1024 * 1024 * 1024))];
    let evict = vec![1];

    let stack = KubernetesStack::new().with_resources(Box::new(FakeResourceRecommender {
        requests: requests.clone(),
        evict: evict.clone(),
    }));

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![ServiceObservation::default()],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut stack = stack;
    let actions = stack.act(&observation);

    assert!(actions.replicas.is_empty());
    assert_eq!(actions.requests, requests);
    assert_eq!(actions.evict, evict);
    assert!(actions.pool_delta.is_empty());
    assert!(actions.drain.is_empty());
}

#[test]
fn kubernetes_stack_only_nodes() {
    let pool_delta = vec![2, 1];
    let drain = vec![NodeId(5), NodeId(10)];

    let stack = KubernetesStack::new().with_nodes(Box::new(FakeNodeProvisioner {
        pool_delta: pool_delta.clone(),
        drain: drain.clone(),
    }));

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![ServiceObservation::default()],
        pools: vec![PoolObservation::default(), PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut stack = stack;
    let actions = stack.act(&observation);

    assert!(actions.replicas.is_empty());
    assert!(actions.requests.is_empty());
    assert!(actions.evict.is_empty());
    assert_eq!(actions.pool_delta, pool_delta);
    assert_eq!(actions.drain, drain);
}

#[test]
fn kubernetes_stack_builder_methods() {
    let stack = KubernetesStack::new()
        .with_replicas(Box::new(FakeReplicaRecommender {
            replicas: vec![Some(4)],
        }))
        .with_resources(Box::new(FakeResourceRecommender {
            requests: vec![],
            evict: vec![],
        }))
        .with_nodes(Box::new(FakeNodeProvisioner {
            pool_delta: vec![],
            drain: vec![],
        }));

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![ServiceObservation::default()],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut stack = stack;
    let actions = stack.act(&observation);

    // Should have replicas from the first component
    assert_eq!(actions.replicas, vec![Some(4)]);
}
