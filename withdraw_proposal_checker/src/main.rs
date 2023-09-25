use crate::gosh::burn::find_burns;
use common::gosh::helper::create_client;
use std::env;
use common::helper::tracing::init_default_tracing;

mod gosh;
mod proposal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    init_default_tracing();
    let context = create_client()?;
    let root_address = env::var("ROOT_ADDRESS")?;
    let burns = find_burns(&context, &root_address).await?;
    println!("burns: {burns:?}");
    Ok(())
}
