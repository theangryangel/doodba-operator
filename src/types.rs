use std::collections::BTreeMap;

use kube::CustomResource;

use k8s_openapi::api::core::v1::{ConfigMapKeySelector, ResourceRequirements, SecurityContext, PodSecurityContext, NodeSelector, Affinity};
use k8s_openapi::api::core::v1::{ObjectReference, Volume, VolumeMount, EnvVar};
use k8s_openapi::api::core::v1::SecretKeySelector;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

type Annotations = Option<BTreeMap<String, String>>;

/// OdooSpec defines the desired state of Odoo
#[derive(CustomResource, Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "doodba.glo.systems",
    version = "v1",
    kind = "Doodba",
    shortname = "Doodba",
    namespaced
)]
#[kube(status = "DoodbaStatus")]
#[serde(rename_all = "camelCase")]
pub struct DoodbaSpec {
    /// The container image to run
    pub image: Option<String>,

    /// The docker tag for the image
    pub version: Option<String>,

    /// The image pull policy
    pub image_pull_policy: Option<String>,

    /// The database credentials
    pub database: Option<Database>,

    /// The filestore storage
    pub filestore: Option<FileStore>,

    /// Extra volumes
    pub extra_volumes: Option<Volume>,

    /// Extra volume mounts
    pub extra_volume_mounts: Option<VolumeMount>,

    /// Extra environment variables
    pub extra_env: Vec<EnvVar>,

    /// If click-odoo-upgrade is enabled
    pub upgrade: bool,

    /// Configuration
    pub config: Option<OdooConfig>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OdooConfig {
    pub without_demo: bool,
    pub list_database: bool,
    pub db_filter: Option<String>,
    pub admin_password: Option<SecretKeySelector>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq, Eq)]
pub enum InstallationState {
    Install,
    Upgrade,
    Running,
}

impl Default for InstallationState {
    fn default() -> Self {
        InstallationState::Install
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DoodbaStatus {
    /// State
    #[serde(default)]
    pub state: InstallationState,

    /// A list of pointers to currently running jobs.
    #[serde(default)]
    pub active: Vec<ObjectReference>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Database {
    pub host: Option<ConfigMapKeySelector>,
    pub port: Option<ConfigMapKeySelector>,
    pub username: Option<SecretKeySelector>,
    pub password: Option<SecretKeySelector>,
    pub database: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileStore {
    /// Access mode
    pub access_modes: Option<Vec<String>>,
    /// Size
    pub size: Option<k8s_openapi::apimachinery::pkg::api::resource::Quantity>,

    /// Storage Class Name
    pub storage_class_name: Option<String>,

    /// Existing claim
    pub existing_claim: Option<Vec<String>>,

    /// Annotations
    pub annotations: Annotations,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Web {
    /// Enabled?
    pub enabled: bool,

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
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JobQueue {
    /// Enabled?
    pub enabled: bool,

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
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Ingress {
    /// Enabled?
    pub enabled: bool,

    /// Hosts
    pub hosts: Vec<String>,

    /// Annotations
    pub annotations: Annotations,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Scheduling {
    /// Resources
    pub resources: Option<ResourceRequirements>,

    /// Node Selector
    pub node_selector: Option<NodeSelector>,

    /// Affinity
    pub affinity: Option<Affinity>,
}

