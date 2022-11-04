mod error;
mod reconcile;
mod types;
mod util;

use std::sync::Arc;

use anyhow::Result;
use futures_util::stream::StreamExt;

use kube::{
    api::{Api, ListParams},
    client::Client,
    runtime::controller::Controller,
    CustomResourceExt,
};

use clap::{Parser, Subcommand};

#[derive(Subcommand)]
enum Command {
    Crd,
    Run,
}

#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    cmd: Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let opts = Opts::parse();
    match opts.cmd {
        Command::Crd => println!("{}", serde_yaml::to_string(&types::Doodba::crd())?),

        Command::Run => {
            let client = Client::try_default().await?;

            let context = Arc::new(reconcile::Data {
                client: client.clone(),
            });

            let docs = Api::<types::Doodba>::all(client);
            // Ensure CRD is installed before loop-watching
            let _r = docs
                .list(&ListParams::default().limit(1))
                .await
                .expect("is the crd installed?"); // FIXME

            Controller::new(docs, ListParams::default())
                .shutdown_on_signal()
                .run(reconcile::reconcile, reconcile::error_policy, context)
                .for_each(|res| async move {
                    match res {
                        Ok(o) => tracing::info!("reconciled {:?}", o),
                        Err(e) => tracing::warn!("reconcile failed: {}", e),
                    }
                })
                .await;

            tracing::info!("controller terminated");
        }
    }

    Ok(())
}
