use common::helper::tracing::init_default_tracing;

mod deposit;

use crate::deposit::check_proposals;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    init_default_tracing();
    check_proposals().await
}
