use common::helper::tracing::init_default_tracing;

mod eth;
mod gosh;
mod proposal_checker;

use crate::proposal_checker::check_proposals;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    init_default_tracing();
    check_proposals().await
}