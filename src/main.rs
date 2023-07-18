mod controller;
mod error;
mod types;
mod util;

use anyhow::Result;

use kube::CustomResourceExt;

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
            controller::run().await?;
        }
    }

    Ok(())
}
