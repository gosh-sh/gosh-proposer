use crate::telemetry::get_telemetry;
use common::helper::tracing::init_default_tracing;

mod telemetry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    init_default_tracing();
    get_telemetry().await
}
