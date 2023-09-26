use crate::checker::check_proposals_and_accept;
use common::helper::tracing::init_default_tracing;

mod checker;
mod eth;
mod gosh;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    init_default_tracing();
    check_proposals_and_accept().await
}
