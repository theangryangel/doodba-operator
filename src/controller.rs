use futures;
use futures_util::stream::StreamExt;
use k8s_openapi::api::{apps::v1::Deployment, batch::v1::Job};
use std::sync::Arc;
use tokio::time::Duration;

use kube::{
    api::{Api, ListParams},
    client::Client,
    runtime::{
        controller::{Action, Controller},
        finalizer::{finalizer, Event as FinalizerEvent},
        watcher::Config,
    },
    ResourceExt,
};

use crate::{error::Error, types};

pub struct Context {
    pub client: kube::Client,
    // TODO
    //pub metrics: Metrics,
}

pub async fn run() -> anyhow::Result<()> {
    let client = Client::try_default()
        .await
        .expect("failed to create kube Client");

    let docs = Api::<types::Doodba>::all(client.clone());

    if let Err(e) = docs.list(&ListParams::default().limit(1)).await {
        tracing::error!("CRD is not queryable; {e:?}. Is the CRD installed?");
        std::process::exit(1);
    }

    let context = Arc::new(Context {
        client: client.clone(),
    });

    Controller::new(docs, Config::default().any_semantic())
        .owns(Api::<Deployment>::all(client.clone()), Config::default())
        .owns(Api::<Job>::all(client.clone()), Config::default())
        .shutdown_on_signal()
        .run(reconcile, error_policy, context)
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_| futures::future::ready(()))
        .await;

    tracing::info!("controller terminated");

    Ok(())
}

pub async fn reconcile(doc: Arc<types::Doodba>, ctx: Arc<Context>) -> Result<Action, Error> {
    let client = ctx.client.clone();
    let ns = doc.namespace().unwrap();
    let docs: Api<types::Doodba> = Api::namespaced(client, &ns);

    tracing::info!("Reconciling Document \"{}\" in {}", doc.name_any(), ns);

    finalizer(&docs, types::FINALIZER, doc, |event| async {
        match event {
            FinalizerEvent::Apply(doc) => doc.reconcile(ctx.clone()).await,
            FinalizerEvent::Cleanup(doc) => doc.cleanup(ctx.clone()).await,
        }
    })
    .await
    .map_err(Error::FinalizerError)
}

/// The controller triggers this on reconcile errors
pub fn error_policy(_object: Arc<types::Doodba>, error: &Error, _ctx: Arc<Context>) -> Action {
    tracing::warn!("reconcile failed: {:?}", error);
    // TODO
    //ctx.metrics.reconcile_failure(&doc, error);
    Action::requeue(Duration::from_secs(5 * 60))
}
