use ksim_core::{actions::Actions, autoscaler::Autoscaler, observation::Observation};

use crate::traits::{NodeProvisioner, ReplicaRecommender, ResourceRecommender};

/// The default Kubernetes autoscaling stack: replicas from the HPA, requests and evictions
/// from the VPA, pool deltas and drains from the Cluster Autoscaler; absent components leave
/// their part of `Actions` empty. As upstream warns, an HPA and a VPA both acting on CPU for
/// the same service interfere.
#[derive(Debug, Default)]
pub struct KubernetesStack {
    replicas: Option<Box<dyn ReplicaRecommender>>,
    resources: Option<Box<dyn ResourceRecommender>>,
    nodes: Option<Box<dyn NodeProvisioner>>,
}

impl KubernetesStack {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_replicas(mut self, recommender: Box<dyn ReplicaRecommender>) -> Self {
        self.replicas = Some(recommender);
        self
    }

    #[must_use]
    pub fn with_resources(mut self, recommender: Box<dyn ResourceRecommender>) -> Self {
        self.resources = Some(recommender);
        self
    }

    #[must_use]
    pub fn with_nodes(mut self, provisioner: Box<dyn NodeProvisioner>) -> Self {
        self.nodes = Some(provisioner);
        self
    }
}

impl Autoscaler for KubernetesStack {
    fn act(&mut self, observation: &Observation) -> Actions {
        let _ = (observation, &self.replicas, &self.resources, &self.nodes);
        todo!()
    }
}
