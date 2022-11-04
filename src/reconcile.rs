use std::sync::Arc;
use tokio::time::Duration;

use crate::{
    error::Error,
    types::{Doodba, FINALIZER},
};
use kube::{
    runtime::{
        controller::Action,
        finalizer::{finalizer, Event as FinalizerEvent},
    },
    Api, ResourceExt,
};

pub struct Data {
    pub client: kube::Client,
}

pub async fn reconcile(doc: Arc<Doodba>, ctx: Arc<Data>) -> Result<Action, Error> {
    let client = ctx.client.clone();
    //let name = doc.name_any();
    let ns = doc.namespace().unwrap();
    let docs: Api<Doodba> = Api::namespaced(client, &ns);

    finalizer(&docs, FINALIZER, doc, |event| async {
        match event {
            FinalizerEvent::Apply(doc) => doc.reconcile(ctx.clone()).await,
            FinalizerEvent::Cleanup(doc) => doc.cleanup(ctx.clone()).await,
        }
    })
    .await
    .map_err(Error::FinalizerError)

    // let mut odoo = (*doodba).clone();
    //
    // let client = &ctx.client;
    //
    // if odoo.spec.suspend {
    //     // FIXME update phase to Suspended - this will current break the state machine
    //     // guess we should use a condition really
    //     return Ok(Action::await_change());
    // }
    //
    //
    // // conditions
    // //  NeedsCreate: True = run before_create hook
    // //  NeedsUpgrade: True = upgrade needed
    // //  Ready: True = NeedsCreate and NeedsUpgrade both != True
    //
    // if odoo.status.is_none() {
    //     // set a default status
    //     let status = DoodbaStatus::default();
    //
    //     client
    //         .apply_patch_status(FIELD_MANAGER_SCOPE, &odoo, &status)
    //         .await?;
    //
    //     odoo.status = Some(status);
    // }
    //
    // // unwrap should be OK here because we've checked above
    // let status = &odoo.status.as_ref().unwrap();
    //
    // // create/update dependant objects
    // // i.e. secrets, configmaps, etc.
    // // exclude deployments, services
    //
    // match status.phase {
    //     DoodbaPhase::Pending => {
    //         if odoo.spec.before_create.is_some() {
    //             // set the phase to creating
    //             client
    //                 .apply_patch_status(
    //                     FIELD_MANAGER_SCOPE,
    //                     &odoo,
    //                     &status.transition_into_creating(),
    //                 )
    //                 .await?;
    //         } else {
    //             // patch directly to running
    //             client
    //                 .apply_patch_status(
    //                     FIELD_MANAGER_SCOPE,
    //                     &odoo,
    //                     &status.transition_into_running(),
    //                 )
    //                 .await?;
    //         }
    //     }
    //
    //     DoodbaPhase::Creating => {

    //
    //         let job_exists = client
    //             .get_opt::<Job>(
    //                 &odoo.before_create_job_name(),
    //                 odoo.namespace().as_deref().unwrap(), // FIXME
    //             )
    //             .await?; // FIXME
    //
    //         if let Some(job) = job_exists {
    //
    //             let conditions = job
    //                 .status
    //                 .as_ref()
    //                 .and_then(|status| status.conditions.clone())
    //                 .unwrap_or_default();
    //
    //                 if conditions
    //                     .iter()
    //                     .any(|condition| condition.type_ == "Failed" && condition.status == "True")
    //                 {
    //                     // mark as failed?
    //                     client
    //                         .apply_patch_status(
    //                             FIELD_MANAGER_SCOPE,
    //                             &odoo,
    //                             &status.transition_into_failed(),
    //                         )
    //                         .await?;
    //                 } else if conditions
    //                     .iter()
    //                     .any(|condition| condition.type_ == "Complete" && condition.status == "True")
    //                 {
    //                     // complete, move to the next state
    //                     client
    //                         .apply_patch_status(
    //                             FIELD_MANAGER_SCOPE,
    //                             &odoo,
    //                             &status.transition_into_running(),
    //                         )
    //                         .await?;
    //                 } else {
    //                     // in progress
    //                 }
    //
    //             // check job status
    //         } else {
    //             // create job
    //             let job = &odoo.build_before_create_job()?;
    //             client.apply_patch(FIELD_MANAGER_SCOPE, job, job).await?;
    //             return Ok(Action::await_change());
    //         }
    //
    //         // check if there's a current job in status.installation:
    //         //  if not create one. requeue.
    //         //  if yes is it running? requeue.
    //         //  if yes, did it complete successfully? set phase to running and requeue.
    //         //  if yes, and it did not complete successfully, log error and retry.
    //     }
    //
    //     DoodbaPhase::Upgrading => {
    //         // any active queue deployments? scale them down.
    //         // any active web deployments where replicaCount > 1, scale to 1.
    //         //
    //         // if waiting on any scaling. requeue.
    //         //
    //         // check if any active upgrade in status.upgrade:
    //         //  if not create one. requeue.
    //         //  if yes, is it running? requeue.
    //         //  if yes, did it complete successfully? set phase to running and requeue.
    //         //  if yes and it did not complete successfully, log error and retry.
    //     }
    //
    //     DoodbaPhase::Running => {
    //     }
    //
    //     _ => {
    //         // unhandled state, await_change
    //     }
    // }
}

/// The controller triggers this on reconcile errors
pub fn error_policy(_object: Arc<Doodba>, _error: &Error, _ctx: Arc<Data>) -> Action {
    Action::requeue(Duration::from_secs(1))
}
