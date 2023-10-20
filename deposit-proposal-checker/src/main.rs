use common::helper::tracing::init_default_tracing;

mod deposit;

use crate::deposit::check_proposals;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load env variables from '.env' file
    dotenv::dotenv().ok();
    // Init tracing in level specified with env 'GOSH_LOG' or "info" level by default
    init_default_tracing();
    // Find existing proposals and check them
    check_proposals().await
}
