use common::helper::tracing::init_default_tracing;
use crate::checker::check_proposals_and_accept;
use crate::eth::proposal::{create_proposal, get_proposals, vote_for_withdrawal};

mod gosh;
mod checker;
mod eth;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    init_default_tracing();
    check_proposals_and_accept().await
}
