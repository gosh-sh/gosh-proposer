mod eth;
mod gosh;
mod helper;

use crate::eth::read_eth_blocks;
use crate::helper::tracing::init_default_tracing;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    init_default_tracing();
    read_eth_blocks().await
}
