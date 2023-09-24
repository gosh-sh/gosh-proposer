use common::eth::block::FullBlock;
use common::eth::encoder::serialize_block;
use common::eth::helper::get_signatures_table;
use common::eth::transfer::filter_and_decode_block_transactions;
use common::gosh::call_function;
use common::gosh::helper::{create_client, load_keys};
use serde_json::json;
use std::env;
use web3::transports::WebSocket;
use web3::types::H256;
use web3::Web3;

const CHECKER_ABI_PATH: &str = "contracts/l2/checker.abi.json";
const KEY_PATH: &str = "tests/keys.json";

pub async fn propose_blocks(
    web3s: &Web3<WebSocket>,
    blocks: Vec<FullBlock<H256>>,
) -> anyhow::Result<()> {
    let checker_address = env::var("CHECKER_ADDRESS")?;
    let client = create_client()?;
    let key_pair = load_keys(KEY_PATH)?;

    // ELOCK contract address
    let eth_contract_address = env::var("ETH_CONTRACT_ADDRESS")?.to_lowercase();

    // Lookup table of contract methods
    let code_sig_lookup = get_signatures_table()?;

    let mut all_transfers = vec![];
    let mut json_blocks = vec![];
    for block in blocks {
        let mut transfers = filter_and_decode_block_transactions(
            web3s,
            &block,
            &eth_contract_address,
            &code_sig_lookup,
        )
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
        &client,
        &checker_address,
        CHECKER_ABI_PATH,
        Some(key_pair),
        "checkData",
        Some(args),
    )
    .await?;
    Ok(())
}
