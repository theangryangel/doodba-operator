use std::sync::Arc;
use tokio::time::Duration;
use kube::runtime::controller::Action;
use thiserror::Error;

use crate::types::Doodba;

pub struct Data {
    pub client: kube::Client,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to serialize status: {0}")]
    SerializeStatusFailed(serde_json::Error),

    #[error("Failed to patch status: {0}")]
    PatchStatusFailed(#[source] kube::Error),

    #[error("Failed to delete active job: {0}")]
    DeleteJobFailed(#[source] kube::Error),

    #[error("Failed to create job: {0}")]
    CreateJobFailed(#[source] kube::Error),
}

pub async fn reconcile(
    doodba: Arc<Doodba>,
    ctx: Arc<Data>,
) -> Result<Action, Error> {

    let mut odoo = (*doodba).clone();
    let client = &ctx.client;

    // TODO implement the damn thing

    Ok(Action::await_change())
}

/// The controller triggers this on reconcile errors
pub fn error_policy(_object: Arc<Doodba>, _error: &Error, _ctx: Arc<Data>) -> Action {
    Action::requeue(Duration::from_secs(1))
}
