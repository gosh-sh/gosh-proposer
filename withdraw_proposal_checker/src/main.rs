use crate::checker::{check_proposals_and_accept, create_new_proposal, get_last_blocks};
use crate::gosh::burn::find_all_burns;
use common::helper::tracing::init_default_tracing;
use std::env;

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
            } else if args[1] == "create" {
                tracing::info!("Create proposal");
                create_new_proposal().await
            } else if args[1] == "find_burns" {
                tracing::info!("Find burns");
                find_all_burns().await
            } else {
                anyhow::bail!("Unknown subcommand");
            }
        }
        _ => check_proposals_and_accept().await,
    }
}
