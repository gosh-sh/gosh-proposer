use common::elock::{get_elock_address, get_last_gosh_block_id};
use common::eth::create_web3_socket;
use common::gosh::block::{get_latest_master_block, get_master_block_seq_no};
use common::gosh::burn::find_burns;
use common::gosh::helper::create_client;

use serde_json::json;

pub async fn find_all_burns() -> anyhow::Result<()> {
    tracing::info!("Find all burns");
    // Create client for GOSH
    let context = create_client()?;

    // Create client for ETH
    let web3s = create_web3_socket().await?;

    // Load ELock address
    let elock_address = get_elock_address()?;

    // Get saved block from ELock
    let first_block = get_last_gosh_block_id(elock_address, &web3s).await?;
    // Get seq no of the saved block
    let first_seq_no = get_master_block_seq_no(&context, &first_block)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get seq no for block from ETH: {e}"))?;

    // Get last block
    let current_master_block = get_latest_master_block(&context)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get latest GOSH block: {e}"))?;

    tracing::info!(
        "master blocks seq no range: {first_seq_no} - {}",
        current_master_block.seq_no
    );

    // Find burns for the specified period of blocks
    let burns = find_burns(&context, first_seq_no, current_master_block.seq_no).await?;
    tracing::info!("burns: {burns:?}");

    // Count total burns number and common value
    let burns_cnt = burns.len();
    let mut total_value = 0;
    for burn in burns {
        total_value += burn.value;
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "count": burns_cnt,
            "total_value": total_value
        }))
        .map_err(|e| anyhow::format_err!("Failed to serialize result: {e}"))?
    );
    Ok(())
}
