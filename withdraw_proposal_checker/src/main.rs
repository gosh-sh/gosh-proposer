use common::helper::tracing::init_default_tracing;
use crate::eth::proposal::create_proposal;

mod gosh;
mod proposal;
mod eth;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    init_default_tracing();
    create_proposal().await?;

    Ok(())
}
