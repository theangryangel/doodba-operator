pub mod annotations;
pub mod config;
pub mod database;
pub mod filestore;
pub mod instance;
pub mod phase;

use std::{sync::Arc, time::Duration};

use k8s_openapi::api::{
    apps::v1::Deployment,
    batch::v1::Job,
    core::v1::{
        ConfigMap, Container, EnvVar, PersistentVolumeClaim, Secret, Service, Volume, VolumeMount,
    },
    networking::v1::Ingress,
};
use kube::{
    api::{Api, Patch, PatchParams, ResourceExt},
    runtime::controller::Action,
    CustomResource,
};

use schemars::JsonSchema;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{error::Error, reconcile::Data};

pub static FINALIZER: &str = "doodba.glo.systems";

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DoodbaStatus {
    /// Phase
    #[serde(default)]
    pub phase: phase::DoodbaPhase,

    // Not using the standard conditions here by design, for now:
    // see https://github.com/kube-rs/kube/issues/43
    // waiting to see how https://github.com/kube-rs/kube/issues/427 shakes out
    /// do we need to run before_create?
    pub needs_create: bool,

    // do we need to wait for a before_update?
    pub needs_update: bool,

    // are we ready to run?
    pub ready: bool,
}

/// OdooSpec defines the desired state of Odoo
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "doodba.glo.systems",
    version = "v1",
    kind = "Doodba",
    plural = "doodbas",
    shortname = "Doodba",
    namespaced
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
    pub database: database::Database,

    /// The filestore storage
    pub filestore: filestore::FileStore,

    /// Extra volumes
    pub extra_volumes: Option<Volume>,

    /// Extra volume mounts
    pub extra_volume_mounts: Option<VolumeMount>,

    /// Extra environment variables
    #[serde(default)]
    pub extra_env: Vec<EnvVar>,

    /// Configuration
    pub config: Option<config::OdooConfig>,

    /// Suspend reconciliation
    #[serde(default)]
    pub suspend: bool,

    /// The deployments to run/execute
    pub instances: Vec<instance::Instance>,

    /// Run a command as a job before create (i.e. init a database using click-odoo)
    pub before_create: Option<String>,

    /// Run a command as a job before update (i.e. upgrade a database using click-odoo)
    pub before_update: Option<String>,
}

impl Doodba {
    async fn patch_status(
        &self,
        api: Api<Doodba>,
        status: DoodbaStatus,
    ) -> Result<(), kube::Error> {
        let new_status = Patch::Apply(json!({
            "apiVersion": "doodba.glo.systems/v1",
            "kind": "Doodba",
            "status": status,
        }));
        let ps = PatchParams::apply("cntrlr").force();
        let name = self.name_any();
        api.patch_status(&name, &ps, &new_status).await?;

        Ok(())
    }

    async fn set_default_status(&self, api: Api<Doodba>) -> Result<(), kube::Error> {
        self.patch_status(
            api,
            DoodbaStatus {
                needs_create: self.spec.before_create.is_some(),
                needs_update: false,
                ready: self.spec.before_create.is_none(),
                ..Default::default()
            },
        )
        .await?;

        Ok(())
    }

    pub async fn reconcile(&self, ctx: Arc<Data>) -> Result<Action, kube::Error> {
        let client = ctx.client.clone();
        let ns = self.namespace().unwrap();
        let api: Api<Doodba> = Api::namespaced(client, &ns);

        if self.status.is_none() {
            self.set_default_status(api).await?;
            return Ok(Action::requeue(Duration::from_secs(1)));
        }

        let status = self.status.as_ref().unwrap();

        if self.spec.suspend {
            let mut status = status.clone();
            status.phase = phase::DoodbaPhase::Suspended;
            status.ready = false;

            self.patch_status(api, status).await?;
            return Ok(Action::await_change());
        }

        // create dependant objects

        if status.needs_create {
            // check if there are any active deployments? if yes, bail out and set the error
            // phase/state
            //
            // check if job exists
            // if not create it
            // if yes, is it finished successfully?
            // change needs_create = false and ready = true
        }

        if status.needs_update {
            // upgrade
            // scale down any active deployments
            // check if job exists
            // if not create it
            // if yes, is it finished successfully?
            // change needs_update = false and ready = true
        }

        // check if requested image != current image
        //  if yes set phase to upgrading. requeue.

        // check if any secrets, etc. changed and rollout restart if replica count = current
        // update any active deployments to the relevant replica count

        // no events, check every 5 minutes
        Ok(Action::requeue(Duration::from_secs(5 * 60)))
    }

    pub async fn cleanup(&self, _ctx: Arc<Data>) -> Result<Action, kube::Error> {
        // todo add some deletion event logging, db clean up, etc.?
        Ok(Action::await_change())
    }

    #[allow(unused)]
    fn before_create_job_name(&self) -> String {
        format!("{}-before-create", self.name_unchecked())
    }

    #[allow(unused)]
    fn before_update_job_name(&self) -> String {
        format!("{}-before-update", self.name_unchecked())
    }

    #[allow(unused)]
    fn configmap_name(&self) -> String {
        format!("{}-config", self.name_unchecked())
    }

    #[allow(unused)]
    fn secret_name(&self) -> String {
        format!("{}-secret", self.name_unchecked())
    }

    #[allow(unused)]
    fn build_before_create_job(&self) -> Result<Job, Error> {
        todo!()

        // if let Some(before_create) = &self.spec.before_create {
        //     let job_meta = ObjectMetaBuilder::new()
        //         .name(self.before_create_job_name())
        //         .namespace_opt(self.namespace())
        //         .ownerreference_from_resource(self, None, Some(true))?
        //         .build();
        //
        //     let container = self
        //         .container_builder("init")?
        //         .command(vec!["/bin/bash".into()])
        //         .args(vec![
        //             "-c".into(),
        //             format!(
        //                 r#"
        //             /opt/odoo/common/entrypoint
        //             {}
        //             "#,
        //                 before_create
        //             )
        //             .into(),
        //         ])
        //         .build();
        //
        //     let pod = PodTemplateSpec {
        //         metadata: Some(
        //             ObjectMetaBuilder::new()
        //                 .name(self.before_create_job_name())
        //                 .build(),
        //         ),
        //         spec: Some(PodSpec {
        //             containers: vec![container],
        //             restart_policy: Some("Never".to_string()),
        //             ..Default::default()
        //         }),
        //     };
        //
        //     let job = Job {
        //         metadata: job_meta,
        //         spec: Some(JobSpec {
        //             template: pod,
        //             ..Default::default()
        //         }),
        //         status: None,
        //     };
        //
        //     return Ok(job);
        // }
        //
        // Err(Error::NoBeforeCreate)
    }

    #[allow(unused)]
    fn build_before_update_job(&self) -> Option<Job> {
        todo!()
    }

    #[allow(unused)]
    fn build_persistentvolumeclaims(&self) -> Vec<PersistentVolumeClaim> {
        todo!()
    }

    #[allow(unused)]
    fn build_secrets(&self) -> Vec<Secret> {
        todo!()
    }

    #[allow(unused)]
    fn build_configmaps(&self) -> Vec<ConfigMap> {
        todo!()
    }

    #[allow(unused)]
    fn build_deployments(&self) -> Vec<Deployment> {
        todo!()
    }

    #[allow(unused)]
    fn build_services(&self) -> Vec<Service> {
        todo!()
    }

    #[allow(unused)]
    fn build_ingress(&self) -> Vec<Ingress> {
        todo!()
    }

    #[allow(unused)]
    fn container_builder(&self, name: &str) -> Container {
        todo!()

        // let configmap = &self.configmap_name();
        // let secret = &self.secret_name();
        //
        // let mut env_vars = vec![
        //     env_var_from_config("PGHOST", configmap, "PGHOST"),
        //     env_var_from_config("PGPORT", configmap, "PGPORT"),
        //     env_var_from_config("PGUSER", configmap, "PGUSER"),
        //     env_var_from_config("PGDATABASE", configmap, "PGDATABASE"),
        //     env_var_from_config("PROXY_MODE", configmap, "PROXY_MODE"),
        //     env_var_from_config("WITHOUT_DEMO", configmap, "WITHOUT_DEMO"),
        //     env_var_from_config("SMTP_SERVER", configmap, "SMTP_SERVER"),
        //     env_var_from_config("SMTP_PORT", configmap, "SMTP_PORT"),
        //     env_var_from_config("SMTP_USER", configmap, "SMTP_USER"),
        //     env_var_from_config("SMTP_SSL", configmap, "SMTP_SSL"),
        //     env_var_from_config("LIST_DB", configmap, "LIST_DB"),
        //     env_var_from_secret("PGPASSWORD", secret, "PGPASSWORD"),
        //     env_var_from_secret("ADMIN_PASSWORD", secret, "ADMIN_PASSWORD"),
        //     env_var_from_secret("SMTP_PASSWORD", secret, "SMTP_PASSWORD"),
        // ];
        //
        // let builder = ContainerBuilder::new(name)?
        //     .image(format!("{}:{}", self.spec.image, self.spec.tag,))
        //     .image_pull_policy(self.spec.image_pull_policy.unwrap_or("IfNotPresent".into()))
        //     .add_env_vars(env_vars);
        //
        // Ok(builder) // FIXME drop the clone
    }
}
