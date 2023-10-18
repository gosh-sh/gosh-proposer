use crate::withdraw::burn::find_all_burns;
use crate::withdraw::validator::{check_proposals_and_accept, create_new_proposal};
use common::eth::events::get_all_events;
use common::helper::get_last_blocks;
use common::helper::tracing::init_default_tracing;
use std::env;

mod withdraw;

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
            } else if args[1] == "events" {
                tracing::info!("Find ELock events");
                get_all_events().await
            } else {
                anyhow::bail!("Unknown subcommand");
            }
        }
        _ => check_proposals_and_accept().await,
    }
}
