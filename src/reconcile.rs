use stackable_operator::k8s_openapi::api::batch::v1::Job;
use stackable_operator::kube::runtime::controller::Action;
use std::sync::Arc;
use tokio::time::Duration;

use crate::{
    error::Error,
    types::{Doodba, DoodbaPhase, DoodbaStatus},
};
use stackable_operator::kube::ResourceExt;

const FIELD_MANAGER_SCOPE: &str = "doodba";

pub struct Data {
    pub client: stackable_operator::client::Client,
}

pub async fn reconcile(doodba: Arc<Doodba>, ctx: Arc<Data>) -> Result<Action, Error> {
    let mut odoo = (*doodba).clone();

    let client = &ctx.client;

    if odoo.spec.suspend {
        // FIXME update phase to Suspended - this will current break the state machine
        // guess we should use a condition really
        return Ok(Action::await_change());
    }

    if odoo.status.is_none() {
        // set a default status
        let status = DoodbaStatus::default();

        client
            .apply_patch_status(FIELD_MANAGER_SCOPE, &odoo, &status)
            .await?;

        odoo.status = Some(status);
    }

    // unwrap should be OK here because we've checked above
    let status = &odoo.status.as_ref().unwrap();

    // create/update dependant objects
    // i.e. secrets, configmaps, etc.
    // exclude deployments, services

    match status.phase {
        DoodbaPhase::Pending => {
            if odoo.spec.before_create.is_some() {
                // create a job, and set the phase to creating
                let job = &odoo.build_before_create_job()?;

                client.apply_patch(FIELD_MANAGER_SCOPE, job, job).await?;

                client
                    .apply_patch_status(
                        FIELD_MANAGER_SCOPE,
                        &odoo,
                        &status.transition_into_creating(),
                    )
                    .await?;
            } else {
                // patch directly to running
                client
                    .apply_patch_status(
                        FIELD_MANAGER_SCOPE,
                        &odoo,
                        &status.transition_into_running(),
                    )
                    .await?;
            }
        }

        DoodbaPhase::Creating => {
            // check if there are any active deployments? if yes, bail out and set the error
            // phase/state

            let job_exists = client
                .get_opt::<Job>(
                    &odoo.before_create_job_name(),
                    odoo.namespace().as_deref().unwrap(), // FIXME
                )
                .await?; // FIXME

            if let Some(_job) = job_exists {
                // check job status
            } else {
                // create job
            }

            // check if there's a current job in status.installation:
            //  if not create one. requeue.
            //  if yes is it running? requeue.
            //  if yes, did it complete successfully? set phase to running and requeue.
            //  if yes, and it did not complete successfully, log error and retry.
        }

        DoodbaPhase::Upgrading => {
            // any active queue deployments? scale them down.
            // any active web deployments where replicaCount > 1, scale to 1.
            //
            // if waiting on any scaling. requeue.
            //
            // check if any active upgrade in status.upgrade:
            //  if not create one. requeue.
            //  if yes, is it running? requeue.
            //  if yes, did it complete successfully? set phase to running and requeue.
            //  if yes and it did not complete successfully, log error and retry.
        }

        DoodbaPhase::Running => {
            // check if requested image != current image
            //  if yes set phase to upgrading. requeue.

            // check if any secrets, etc. changed and rollout restart if replica count = current
            // update any active deployments to the relevant replica count
        }

        _ => {
            // unhandled state, await_change
        }
    }

    Ok(Action::await_change())
}

/// The controller triggers this on reconcile errors
pub fn error_policy(_object: Arc<Doodba>, _error: &Error, _ctx: Arc<Data>) -> Action {
    Action::requeue(Duration::from_secs(1))
}
