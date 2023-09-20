mod eth;
mod gosh;
mod helper;

use crate::eth::read_eth_blocks;
use crate::gosh::{call_function, call_getter};
use crate::gosh::helper::{create_client, load_keys};
use crate::helper::tracing::init_default_tracing;
use std::env;
use serde_json::json;

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

    let res = call_getter(
        &client,
        &contract_address,
        abi_path,
        "getDetails",
        None
    ).await?;
    println!("res = {res:?}");

    let giver_address = "0:ece57bcc6c530283becbbd8a3b24d3c5987cdddc3c8b7b33be6e4a6312490415".to_string();
    let giver_abi = "../gosh/tests/node_se_scripts/local_giver.abi.json";
    let giver_key_path = "../gosh/tests/node_se_scripts/local_giver.keys.json";
    let key_pair = load_keys(giver_key_path)?;

    call_function(
        &client,
        &giver_address,
        giver_abi,
        Some(key_pair),
        "sendTransaction",
        Some(json!({
            "dest": contract_address,
            "value": "10000000000",
            "bounce": false
        }))
    ).await?;

    Ok(())
}
