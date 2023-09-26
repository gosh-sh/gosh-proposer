use std::env;
use crate::checker::{check_proposals_and_accept, get_last_blocks};
use common::helper::tracing::init_default_tracing;

mod checker;
mod eth;
mod gosh;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    init_default_tracing();

    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => {
            if args[1] == "get_last_blocks" {
                tracing::info!("Get last blocks");
                get_last_blocks().await
            } else {
                anyhow::bail!("Unknown subcommand");
            }
        },
        _ => check_proposals_and_accept().await,
    }
}
