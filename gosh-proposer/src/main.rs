mod proposer;

use crate::proposer::propose_eth_blocks;
use common::helper::tracing::init_default_tracing;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    init_default_tracing();
    propose_eth_blocks().await
}
