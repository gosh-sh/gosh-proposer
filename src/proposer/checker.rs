use crate::eth::proof::serialize_block;
use crate::gosh::call_function;
use crate::gosh::helper::{create_client, load_keys};
use serde_json::json;
use std::env;
use web3::types::{Block, H256};

pub async fn check_blocks(blocks: Vec<Block<H256>>) -> anyhow::Result<()> {
    let checker_address = env::var("CHECKER_ADDRESS")?;
    let client = create_client()?;
    let abi_path = "tests/solidity/checker.abi.json";
    let key_path = "tests/keys.json";
    let key_pair = load_keys(key_path)?;

    let mut json_blocks = vec![];
    for block in blocks {
        let hash = format!("{:?}", block.hash.unwrap());
        let data = serialize_block(block)?;
        let data_str = data
            .iter()
            .fold(String::new(), |acc, el| format!("{acc}{:02x}", el));
        json_blocks.push(json!({"data": data_str, "hash": hash}));
    }
    json_blocks.reverse();
    let args = json!({
        "data": json_blocks
    });
    tracing::info!("args: {args:?}");

    call_function(
        &client,
        &checker_address,
        abi_path,
        Some(key_pair),
        "checkData",
        Some(args),
    )
    .await?;
    Ok(())
}
