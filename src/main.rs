mod eth;
mod gosh;
mod helper;

use crate::eth::read_eth_blocks;
use crate::gosh::call_getter;
use crate::gosh::helper::create_client;
use crate::helper::tracing::init_default_tracing;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    init_default_tracing();
    call_gosh().await?;
    read_eth_blocks().await
}

async fn call_gosh() -> anyhow::Result<()> {
    let client = create_client()?;
    let contract_address = env::var("GOSH_CONTRACT_ADDRESS")?.to_lowercase();
    let abi_path = "resources/gosh_contract.abi.json";

    let res = call_getter(&client, &contract_address, abi_path, "getDetails", None).await?;

    println!("res = {res:?}");
    Ok(())
}
