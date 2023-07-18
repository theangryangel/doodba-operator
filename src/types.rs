pub mod phase;

use std::{collections::BTreeMap, sync::Arc};

use kube::{
    api::{Api, ListParams, Patch, PatchParams, PostParams},
    core::ObjectMeta,
    runtime::controller::Action,
    CustomResource, Resource, ResourceExt,
};

use k8s_openapi::api::{
    batch::v1::{Job, JobSpec},
    core::v1::{Container, PodSpec, PodTemplateSpec},
};

use schemars::JsonSchema;

use crate::controller::Context;
use serde::{Deserialize, Serialize};

use self::phase::DoodbaPhase;

pub static FINALIZER: &str = "doodba.glo.systems";

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DoodbaStatus {
    /// Phase
    #[serde(default)]
    pub phase: phase::DoodbaPhase,

    #[serde(default)]
    pub last_applied_image: Option<String>,

    /// Current state
    #[serde(default)]
    pub initialised: bool,
}

/// OdooSpec defines the desired state of Odoo
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "doodba.glo.systems",
    version = "v1alpha1",
    kind = "Doodba",
    plural = "doodba",
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

    /// Suspend reconciliation
    #[serde(default)]
    pub suspend: bool,
    // bootstrap: options to bootstrap
    //  enabled: bool
    //  // FIXME: this should be safe, this isn't, it will trash dbs
    //  command: "odoo -i base --stop-after-init --no-http"
    // upgrade: options to upgrade
    //  command: "click-odoo-upgrade"

    // postgres: how to configure
    // postgres:
    //  host: "string"
    //  port: "string"
    //  username: from secret ref
    //  password: from secret ref

    // filestore pvc

    // volumes: list of extra volumes
    // volumeMounts: list of extra volumeMounts
    // env: list of extra Envs

    // web configuration
    // web:
    //  replica_count: 1
    //  volumes: []
    //  volumeMounts: []
    //  env: []
    //  // FIXME: Can we automate this somehow?
    //  websocket: true
    //  longpolling: true
    //  // list of domains?
    //  ingress: []
    //
    // queue configuration
    // queue:
    //  replica_count: 1
    //  volumes: []
    //  volumeMounts: []
    //  env: []

    // TODO how do we configure different instances without much effort?
    // ie. we need to be be able to point QUEUE_JOB at different postgreSQL endpoints.
    // TODO how do we configure the differences between 16.0 and prior (i.e. /websocket vs
    // /longpolling/poll)?
}

impl DoodbaSpec {
    pub fn full_image_name(&self) -> String {
        format!("{}:{}", self.image, self.tag)
    }
}

impl Doodba {
    pub async fn reconcile(&self, ctx: Arc<Context>) -> Result<Action, kube::Error> {
        let client = ctx.client.clone();
        let ns = self.namespace().unwrap();
        let docs: Api<Doodba> = Api::namespaced(client, &ns);

        let mut status = self.status.clone().unwrap_or_default();

        // are we suspended?
        if self.spec.suspend {
            status.phase = DoodbaPhase::Suspended;
            self.patch_status(&docs, status).await?;

            return Ok(Action::await_change());
        }

        // should we run the init container?
        if !status.initialised {
            return self.reconcile_initialization(docs, ctx.clone()).await;
        }

        // if image != current image upgrade
        let should_upgrade = if_chain::if_chain! {
            if let Some(image) = status.last_applied_image;
            if image != self.spec.full_image_name();
            then {
                true
            } else {
                false
            }
        };

        if should_upgrade {
            todo!()
            // scale down web worker
            // scale down job queue
            // scale down long polling
            // execute upgrade
        }

        // scale everything up
        self.reconcile_secrets(ctx.clone()).await?;
        self.reconcile_configmaps(ctx.clone()).await?;
        self.reconcile_services(ctx.clone()).await?;
        self.reconcile_deployments(1, ctx.clone()).await?;
        self.reconcile_ingresses(ctx.clone()).await?;

        // no events, check every 5 minutes
        // Ok(Action::requeue(Duration::from_secs(5 * 60)))
        Ok(Action::await_change())
    }

    pub async fn cleanup(&self, _ctx: Arc<Context>) -> Result<Action, kube::Error> {
        // todo add some deletion event logging, db clean up, etc.?
        Ok(Action::await_change())
    }

    async fn patch_status(
        &self,
        doc: &Api<Doodba>,
        status: DoodbaStatus,
    ) -> Result<(), kube::Error> {
        let patch = serde_json::json!({
            "apiVersion": "doodba.glo.systems/v1alpha1",
            "kind": "Doodba",
            "status": status,
        });

        let ps = PatchParams::apply("cntrlr").force();
        let patch_status = Patch::Apply(patch);
        doc.patch_status(&self.name_any(), &ps, &patch_status)
            .await?;
        Ok(())
    }

    async fn reconcile_initialization(
        &self,
        docs: Api<Doodba>,
        ctx: Arc<Context>,
    ) -> Result<Action, kube::Error> {
        // check if init container exists
        let ns = self.namespace().unwrap();
        let job = self.build_initialization_job();
        let job_name = job.metadata.name.clone().unwrap();

        let mut status = self.status.clone().unwrap_or_default();

        let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ns);
        match jobs.get(&job_name).await {
            Ok(job) => {
                let job_status = job.status.unwrap_or_default();

                if !(job_status.succeeded.unwrap_or_default() > 0
                    && job_status.active.unwrap_or_default() == 0)
                {
                    // wait for the jobs to finish and succeed
                    // owner_references will handle this for us.
                    return Ok(Action::await_change());
                }

                // set complete and move on
                status.initialised = true;
                status.phase = DoodbaPhase::Ready;
                self.patch_status(&docs, status.clone()).await?;
                // TODO: Should this be a retry in X seconds?
                return Ok(Action::await_change());
            }
            Err(_) => {
                // doesn't exist, create it
                jobs.create(&PostParams::default(), &job).await?;

                status.initialised = false;
                status.phase = DoodbaPhase::Initializing;
                self.patch_status(&docs, status.clone()).await?;

                // owner_references will trigger for us later.
                return Ok(Action::await_change());
            }
        }
    }

    fn build_initialization_job(&self) -> Job {
        let ns = self.namespace().unwrap();
        let name = self.name_any();
        let oref = self.controller_owner_ref(&()).unwrap();
        let job_name = format!("{}-init", self.name_any());

        let mut labels: BTreeMap<String, String> = BTreeMap::new();
        labels.insert("app".to_owned(), "doodba".to_string());
        labels.insert("doodba.glo.systems/name".to_owned(), name.to_owned());
        labels.insert("init".to_owned(), name.to_owned());

        Job {
            metadata: ObjectMeta {
                name: Some(job_name),
                namespace: Some(ns),
                labels: Some(labels.clone()),
                owner_references: Some(vec![oref]),
                ..Default::default()
            },
            spec: Some(JobSpec {
                template: PodTemplateSpec {
                    spec: Some(PodSpec {
                        containers: vec![Container {
                            name: "init".to_string(),
                            image: Some("hello-world:latest".to_string()),
                            ..Container::default()
                        }],
                        restart_policy: Some("OnFailure".to_string()),
                        ..PodSpec::default()
                    }),
                    ..PodTemplateSpec::default()
                },
                ..JobSpec::default()
            }),
            ..Default::default()
        }
    }

    async fn reconcile_secrets(&self, ctx: Arc<Context>) -> Result<(), kube::Error> {
        todo!()
    }

    async fn reconcile_configmaps(&self, ctx: Arc<Context>) -> Result<(), kube::Error> {
        todo!()
    }

    async fn reconcile_deployments(
        &self,
        replicas: usize,
        ctx: Arc<Context>,
    ) -> Result<(), kube::Error> {
        todo!()
    }

    async fn reconcile_services(&self, ctx: Arc<Context>) -> Result<(), kube::Error> {
        todo!()
    }

    async fn reconcile_ingresses(&self, ctx: Arc<Context>) -> Result<(), kube::Error> {
        todo!()
    }
}
