mod error;
mod reconcile;
mod types;

use std::sync::Arc;

use anyhow::Result;
use futures_util::stream::StreamExt;
use stackable_operator::kube::api::ListParams;
use stackable_operator::kube::runtime::Controller;
use stackable_operator::kube::CustomResourceExt;

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
            let client =
                stackable_operator::client::create_client(Some("odoo.glo.systems".into())).await?;

            Controller::new(client.get_all_api::<types::Doodba>(), ListParams::default())
                .shutdown_on_signal()
                .run(
                    reconcile::reconcile,
                    reconcile::error_policy,
                    Arc::new(reconcile::Data { client }),
                )
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
