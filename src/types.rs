use std::collections::BTreeMap;

use stackable_operator::k8s_openapi::api::apps::v1::Deployment;
use stackable_operator::k8s_openapi::api::batch::v1::{Job, JobSpec};
use stackable_operator::k8s_openapi::api::networking::v1::Ingress;
use stackable_operator::k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use stackable_operator::kube::{CustomResource, ResourceExt};
use stackable_operator::schemars::{self, JsonSchema};

use stackable_operator::k8s_openapi::api::core::v1::{
    Affinity, ConfigMap, ConfigMapKeySelector, ContainerPort, NodeSelector, PersistentVolumeClaim,
    PodSecurityContext, PodSpec, PodTemplateSpec, ResourceRequirements, Secret, SecurityContext,
    Service,
};
use stackable_operator::k8s_openapi::api::core::v1::{EnvVar, EnvVarSource, SecretKeySelector};
use stackable_operator::k8s_openapi::api::core::v1::{ObjectReference, Volume, VolumeMount};

use serde::{Deserialize, Serialize};
use stackable_operator::builder::{ContainerBuilder, ObjectMetaBuilder};

use crate::error::Error;

type Annotations = Option<BTreeMap<String, String>>;

/// OdooSpec defines the desired state of Odoo
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "doodba.glo.systems",
    version = "v1",
    kind = "Doodba",
    plural = "doodbas",
    shortname = "Doodba",
    namespaced,
    crates(
        kube_core = "stackable_operator::kube::core",
        k8s_openapi = "stackable_operator::k8s_openapi",
        schemars = "stackable_operator::schemars"
    )
)]
#[kube(status = "DoodbaStatus")]
#[serde(rename_all = "camelCase")]
pub struct DoodbaSpec {
    /// The container image to run
    pub image: String,

    /// The docker tag for the image
    pub tag: String,

    /// The image pull policy
    pub image_pull_policy: Option<String>,

    /// The database credentials
    pub database: Database,

    /// The filestore storage
    pub filestore: FileStore,

    /// Extra volumes
    pub extra_volumes: Option<Volume>,

    /// Extra volume mounts
    pub extra_volume_mounts: Option<VolumeMount>,

    /// Extra environment variables
    pub extra_env: Vec<EnvVar>,

    /// Configuration
    pub config: Option<OdooConfig>,

    /// Suspend reconciliation
    #[serde(default)]
    pub suspend: bool,

    /// The deployments to run/execute
    pub instances: Vec<Instance>,

    /// Run a command as a job before create (i.e. init a database using click-odoo)
    pub before_create: Option<String>,

    /// Run a command as a job before update (i.e. upgrade a database using click-odoo)
    pub before_update: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OdooConfig {
    pub without_demo: bool,
    pub list_database: bool,
    pub db_filter: Option<String>,
    pub admin_password: Option<SecretKeySelector>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum DoodbaPhase {
    Pending,   // initial state
    Creating,  // Running before_create
    Upgrading, // Running any before_update
    Running,   // healthy
    Failed,    // unhealthy installation
    Suspended,
}

impl Default for DoodbaPhase {
    fn default() -> Self {
        DoodbaPhase::Pending
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DoodbaStatus {
    /// Phase
    #[serde(default)]
    pub phase: DoodbaPhase,

    /// A pointer to currently running installation attempt.
    #[serde(default)]
    pub before_create_job: Option<ObjectReference>,

    /// A pointer to currently running upgrade job.
    #[serde(default)]
    pub before_update_job: Option<ObjectReference>,
}

impl DoodbaStatus {
    pub fn transition_into_pending(&self) -> Self {
        let mut new = self.clone();
        new.phase = DoodbaPhase::Pending;
        new
    }

    pub fn transition_into_creating(&self) -> Self {
        // FIXME set job ref?
        let mut new = self.clone();
        new.phase = DoodbaPhase::Creating;
        new
    }

    pub fn transition_into_upgrading(&self) -> Self {
        // FIXME set job ref?
        let mut new = self.clone();
        new.phase = DoodbaPhase::Upgrading;
        new
    }

    pub fn transition_into_running(&self) -> Self {
        // FIXME clear old job ref?
        let mut new = self.clone();
        new.phase = DoodbaPhase::Running;
        new
    }

    pub fn transition_into_failed(&self) -> Self {
        // FIXME clear old job ref?
        let mut new = self.clone();
        new.phase = DoodbaPhase::Failed;
        new
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Database {
    pub host: Option<ConfigMapKeySelector>,
    pub port: Option<ConfigMapKeySelector>,
    pub username: Option<SecretKeySelector>,
    pub password: Option<SecretKeySelector>,
    pub database: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileStore {
    /// Access mode
    pub access_modes: Option<Vec<String>>,
    /// Size
    pub size: Option<Quantity>,

    /// Storage Class Name
    pub storage_class_name: Option<String>,

    /// Existing claim
    pub existing_claim: Option<Vec<String>>,

    /// Annotations
    pub annotations: Annotations,
}

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

impl Doodba {
    pub fn before_create_job_name(&self) -> String {
        format!("{}-before-create", self.name_unchecked())
    }

    pub fn before_update_job_name(&self) -> String {
        format!("{}-before-update", self.name_unchecked())
    }

    pub fn configmap_name(&self) -> String {
        format!("{}-config", self.name_unchecked())
    }

    pub fn secret_name(&self) -> String {
        format!("{}-secret", self.name_unchecked())
    }

    pub fn build_before_create_job(&self) -> Result<Job, Error> {
        if let Some(before_create) = &self.spec.before_create {
            let job_meta = ObjectMetaBuilder::new()
                .name(self.before_create_job_name())
                .namespace_opt(self.namespace())
                .ownerreference_from_resource(self, None, Some(true))?
                .build();

            let container = self
                .container_builder("init")?
                .command(vec!["/bin/bash".into()])
                .args(vec![
                    "-c".into(),
                    format!(
                        r#"
                    /opt/odoo/common/entrypoint
                    {}
                    "#,
                        before_create
                    )
                    .into(),
                ])
                .build();

            let pod = PodTemplateSpec {
                metadata: Some(
                    ObjectMetaBuilder::new()
                        .name(self.before_create_job_name())
                        .build(),
                ),
                spec: Some(PodSpec {
                    containers: vec![container],
                    restart_policy: Some("Never".to_string()),
                    ..Default::default()
                }),
            };

            let job = Job {
                metadata: job_meta,
                spec: Some(JobSpec {
                    template: pod,
                    ..Default::default()
                }),
                status: None,
            };

            return Ok(job);
        }

        Err(Error::NoBeforeCreate)
    }

    pub fn build_before_update_job(&self) -> Option<Job> {
        todo!()
    }

    pub fn build_persistentvolumeclaims(&self) -> Vec<PersistentVolumeClaim> {
        todo!()
    }

    pub fn build_secrets(&self) -> Vec<Secret> {
        todo!()
    }

    pub fn build_configmaps(&self) -> Vec<ConfigMap> {
        todo!()
    }

    pub fn build_deployments(&self) -> Vec<Deployment> {
        todo!()
    }

    pub fn build_services(&self) -> Vec<Service> {
        todo!()
    }

    pub fn build_ingress(&self) -> Vec<Ingress> {
        todo!()
    }

    pub fn container_builder(&self, name: &str) -> Result<ContainerBuilder, Error> {
        let configmap = &self.configmap_name();
        let secret = &self.secret_name();

        let mut env_vars = vec![
            env_var_from_config("PGHOST", configmap, "PGHOST"),
            env_var_from_config("PGPORT", configmap, "PGPORT"),
            env_var_from_config("PGUSER", configmap, "PGUSER"),
            env_var_from_config("PGDATABASE", configmap, "PGDATABASE"),
            env_var_from_config("PROXY_MODE", configmap, "PROXY_MODE"),
            env_var_from_config("WITHOUT_DEMO", configmap, "WITHOUT_DEMO"),
            env_var_from_config("SMTP_SERVER", configmap, "SMTP_SERVER"),
            env_var_from_config("SMTP_PORT", configmap, "SMTP_PORT"),
            env_var_from_config("SMTP_USER", configmap, "SMTP_USER"),
            env_var_from_config("SMTP_SSL", configmap, "SMTP_SSL"),
            env_var_from_config("LIST_DB", configmap, "LIST_DB"),
            env_var_from_secret("PGPASSWORD", secret, "PGPASSWORD"),
            env_var_from_secret("ADMIN_PASSWORD", secret, "ADMIN_PASSWORD"),
            env_var_from_secret("SMTP_PASSWORD", secret, "SMTP_PASSWORD"),
        ];

        let mut builder = ContainerBuilder::new(name)?
            .image(format!("{}:{}", self.spec.image, self.spec.tag,))
            .image_pull_policy(self.spec.image_pull_policy.unwrap_or("IfNotPresent".into()))
            .add_env_vars(env_vars);

        if let Some(policy) = self.spec.image_pull_policy {
            builder = builder.image_pull_policy(policy);
        }

        Ok(builder.clone()) // FIXME drop the clone
    }
}

pub fn env_var_from_secret(var_name: &str, secret: &str, secret_key: &str) -> EnvVar {
    EnvVar {
        name: String::from(var_name),
        value_from: Some(EnvVarSource {
            secret_key_ref: Some(SecretKeySelector {
                name: Some(String::from(secret)),
                key: String::from(secret_key),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub fn env_var_from_config(var_name: &str, config: &str, config_key: &str) -> EnvVar {
    EnvVar {
        name: String::from(var_name),
        value_from: Some(EnvVarSource {
            config_map_key_ref: Some(ConfigMapKeySelector {
                name: Some(String::from(config)),
                key: String::from(config_key),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}
