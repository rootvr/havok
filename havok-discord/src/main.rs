use clap::Parser;
use tracing::Level;

mod cli;
mod discord;

#[tokio::main]
#[tracing::instrument]
async fn main() {
    let args = cli::Args::parse();

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_ansi(true)
        .with_max_level(if args.debug {
            Level::DEBUG
        } else {
            Level::INFO
        })
        .init();

    discord::run().await;
}
