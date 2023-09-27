use common::eth::block::FullBlock;
use common::eth::encoder::serialize_block;

use common::eth::transfer::filter_and_decode_block_transactions;
use common::gosh::call_function;
use common::gosh::helper::EverClient;
use serde_json::json;
use std::env;
use std::sync::Arc;
use web3::transports::WebSocket;
use web3::types::H256;
use web3::Web3;

const CHECKER_ABI_PATH: &str = "contracts/l2/checker.abi.json";

pub async fn propose_blocks(
    web3s: Arc<Web3<WebSocket>>,
    client: &EverClient,
    blocks: Vec<FullBlock<H256>>,
) -> anyhow::Result<()> {
    let checker_address = env::var("CHECKER_ADDRESS")?;

    // ELOCK contract address
    let eth_contract_address = env::var("ETH_CONTRACT_ADDRESS")?.to_lowercase();

    let mut all_transfers = vec![];
    let mut json_blocks = vec![];

    // TODO: use eth_getLogs api function instead to get all account transactions

    for block in blocks {
        let mut transfers =
            filter_and_decode_block_transactions(web3s.clone(), &block, &eth_contract_address)
                .await?;
        all_transfers.append(&mut transfers);
        let hash = format!("{:?}", block.hash.unwrap());
        let data = serialize_block(block)?;
        let data_str = data
            .iter()
            .fold(String::new(), |acc, el| format!("{acc}{:02x}", el));
        json_blocks.push(json!({"data": data_str, "hash": hash}));
    }
    json_blocks.reverse();
    let args = json!({
        "data": json_blocks,
        "transactions": all_transfers,
    });

    call_function(
        client,
        &checker_address,
        CHECKER_ABI_PATH,
        None,
        "checkData",
        Some(args),
    )
    .await?;
    Ok(())
}
