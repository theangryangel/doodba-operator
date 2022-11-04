use k8s_openapi::api::core::v1::{
    Affinity, ContainerPort, EnvVar, NodeSelector, PodSecurityContext, ResourceRequirements,
    SecurityContext,
};

use schemars::JsonSchema;

use serde::{Deserialize, Serialize};

use super::annotations::Annotations;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    /// Enabled?
    pub enabled: bool,

    /// Instance name
    pub name: String,

    /// Extra configuration items
    pub extra_config: Option<String>,

    /// Replica count
    pub replicas: usize,

    /// Extra environment variables
    pub extra_env: Vec<EnvVar>,

    /// Security context
    pub security_context: Option<SecurityContext>,

    /// Pod Security Context
    pub pod_security_context: Option<PodSecurityContext>,

    /// Pod Annotations
    pub pod_annotations: Annotations,

    /// Schedulding
    pub scheduling: Option<Scheduling>,

    /// scale down during upgrade?
    pub scale_during_upgrade: bool,

    /// Ports to expose and services for
    #[serde(default)]
    pub ports: Vec<ContainerPort>,

    /// Ingress
    #[serde(default)]
    pub ingress: Vec<InstanceIngress>,

    /// Optional command to run instead of normal entrypoint
    pub command: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InstanceIngress {
    /// Enabled?
    pub enabled: bool,

    /// Hosts
    pub hosts: Vec<String>,

    /// Port
    pub port: i32,

    /// Annotations
    pub annotations: Annotations,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Scheduling {
    /// Resources
    pub resources: Option<ResourceRequirements>,

    /// Node Selector
    pub node_selector: Option<NodeSelector>,

    /// Affinity
    pub affinity: Option<Affinity>,
}
