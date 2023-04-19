mod cli;
mod command;
mod discord;

use clap::Parser;
use tracing::Level;

#[tokio::main]
#[tracing::instrument]
async fn main() {
    let args = cli::Args::parse();

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_ansi(true)
        .with_max_level(if args.verbose {
            Level::DEBUG
        } else {
            Level::INFO
        })
        .init();

    discord::run().await;
}
