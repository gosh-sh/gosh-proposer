mod proposer;

use crate::proposer::propose_eth_blocks;
use common::helper::tracing::init_default_tracing;

#[tokio::main(flavor = "multi_thread", worker_threads = 20)]
async fn main() -> anyhow::Result<()> {
    // Load env variables from '.env' file
    dotenv::dotenv().ok();
    // Init tracing in level specified with env 'GOSH_LOG' or "info" level by default
    init_default_tracing();
    // Propose eth blocks to GOSH
    propose_eth_blocks().await
}
