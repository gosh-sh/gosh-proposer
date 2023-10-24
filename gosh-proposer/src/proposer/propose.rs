use common::elock::deposit::get_deposits;
use common::elock::transfer::TransferPatch;
use common::elock::{get_elock_address, get_tx_counter};
use common::eth::encoder::serialize_block;
use common::eth::FullBlock;
use common::gosh::call_function;
use common::gosh::helper::EverClient;
use common::helper::abi::CHECKER_ABI;
use common::token_root::{deploy_root, is_root_active};
use serde_json::json;
use std::collections::HashSet;
use web3::transports::WebSocket;
use web3::types::H256;
use web3::Web3;

pub async fn propose_blocks(
    web3s: &Web3<WebSocket>,
    client: &EverClient,
    blocks: Vec<FullBlock<H256>>,
    checker_address: &str,
) -> anyhow::Result<()> {
    tracing::info!("start propose block");

    // ELock contract address
    let elock_address = get_elock_address()?;

    // Get starting tx counter
    let start_block_number = blocks.last().unwrap().number.unwrap();
    let starting_tx_counter = get_tx_counter(web3s, elock_address, start_block_number)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get ELock tx counter: {e}"))?;
    tracing::info!("Start tx counter on {start_block_number}: {starting_tx_counter}");
    // Get final tx counter
    let final_block_number = blocks.first().unwrap().number.unwrap();
    let final_tx_counter = get_tx_counter(web3s, elock_address, final_block_number)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get ELock tx counter: {e}"))?;
    tracing::info!("Final tx counter on {final_block_number}: {final_tx_counter}");

    let all_transfers: Vec<TransferPatch> = if final_tx_counter != starting_tx_counter {
        let transfers =
            get_deposits(web3s, elock_address, start_block_number, final_block_number).await?;
        assert_eq!(
            transfers.len(),
            (final_tx_counter - starting_tx_counter).as_usize(),
            "Number of deposits does not match tx counter"
        );
        check_roots(client, checker_address, &transfers).await?;
        transfers
    } else {
        vec![]
    };

    let mut json_blocks = vec![];

    // Iterate through blocks and check whether we need to look for transfers
    for block in blocks.iter().rev() {
        // Format blocks before sending them to the checker contract
        let hash = format!("{:?}", block.hash.unwrap());
        let data = serialize_block(block)
            .map_err(|e| anyhow::format_err!("Failed to serialize ETH block: {e}"))?;
        let data_str = data
            .iter()
            .fold(String::new(), |acc, el| format!("{acc}{:02x}", el));
        json_blocks.push(json!({"data": data_str, "hash": hash}));
    }

    // Send data to the checker contract
    tracing::info!("Send transaction to checker: {all_transfers:?}");
    let args = json!({
        "data": json_blocks,
        "transactions": all_transfers,
    });

    call_function(
        client,
        checker_address,
        CHECKER_ABI,
        None,
        "checkData",
        Some(args),
    )
    .await
    .map_err(|e| anyhow::format_err!("Failed to call GOSH function: {e}"))?;
    Ok(())
}

async fn check_roots(
    gosh_context: &EverClient,
    checker_address: &str,
    transfers: &Vec<TransferPatch>,
) -> anyhow::Result<()> {
    let mut deployed_roots = HashSet::new();
    for transfer in transfers {
        let token_root = web3::helpers::to_string(&transfer.root.eth_root).replace('"', "");
        if !deployed_roots.contains(&token_root) {
            match is_root_active(gosh_context, checker_address, &transfer.root).await {
                Ok(true) => {}
                _ => {
                    deploy_root(gosh_context, checker_address, &transfer.root).await?;
                }
            };
            deployed_roots.insert(token_root);
        }
    }

    Ok(())
}
