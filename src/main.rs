mod eth;
mod gosh;
mod helper;
mod proposer;

use crate::helper::tracing::init_default_tracing;
use crate::proposer::propose_eth_blocks;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    init_default_tracing();
    propose_eth_blocks().await
}
