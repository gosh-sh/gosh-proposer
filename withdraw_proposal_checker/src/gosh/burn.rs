use crate::eth::elock::get_last_gosh_block_id;
use crate::gosh::block::{get_latest_master_block, get_master_block_seq_no};
use crate::gosh::monitor::query_messages;
use common::gosh::helper::{create_client, EverClient};
use common::helper::abi::ROOT_ABI;
use serde::Deserialize;
use serde_json::json;
use std::env;
use std::sync::Arc;
use ton_client::abi::{decode_message_body, Abi, ParamsOfDecodeMessageBody};
use web3::transports::WebSocket;
use web3::Web3;

#[derive(Debug, PartialEq)]
pub struct Burn {
    pub dest: String,
    pub value: u128,
    pub tx_id: String,
}

#[derive(Deserialize)]
struct BurnArguments {
    _answer_id: String,
    #[serde(rename = "pubkey")]
    _pubkey: String,
    #[serde(rename = "owner")]
    _owner: Option<String>,
    tokens: String,
    to: String,
}

pub async fn find_burns(
    context: &EverClient,
    root_address: &str,
    start_seq_no: u128,
    end_seq_no: u128,
) -> anyhow::Result<Vec<Burn>> {
    let messages = query_messages(context, root_address, start_seq_no, end_seq_no)
        .await
        .map_err(|e| anyhow::format_err!("Failed to query messages to ROOT: {e}"))?;

    let abi = Abi::Json(ROOT_ABI.to_string());
    let root_function_name = env::var("ROOT_FUNCTION_NAME")
        .map_err(|e| anyhow::format_err!("Failed to get env ROOT_FUNCTION_NAME: {e}"))?;

    let mut res = vec![];
    for message in messages {
        let body = message.body;
        tracing::info!("decode message: {} {body}", message.id);
        let decode_params = ParamsOfDecodeMessageBody {
            abi: abi.clone(),
            body,
            is_internal: true,
            allow_partial: false,
            function_name: None,
            data_layout: None,
        };
        let decode_result = decode_message_body(Arc::clone(context), decode_params).await;
        if let Ok(decode_result) = decode_result {
            if decode_result.name != root_function_name {
                continue;
            }
            let args: BurnArguments = serde_json::from_value(decode_result.value.unwrap())
                .map_err(|e| anyhow::format_err!("Failed to serialize burn arguments: {e}"))?;
            let trimmed_to = args.to[26..].to_string();
            let dest = format!("0x{}", trimmed_to);
            res.push(Burn {
                dest,
                value: args
                    .tokens
                    .parse::<u128>()
                    .map_err(|e| anyhow::format_err!("Failed to convert tokens to u128: {e}"))?,
                tx_id: message.tx_id,
            })
        } else {
            tracing::info!("Failed to decode message, skip it. ID={}", message.id);
        }
    }
    Ok(res)
}

pub async fn find_all_burns() -> anyhow::Result<()> {
    tracing::info!("Find all burns");
    let context = create_client()?;
    let websocket = WebSocket::new(
        &env::var("ETH_NETWORK")
            .map_err(|e| anyhow::format_err!("Failed to get env ETH_NETWORK: {e}"))?,
    )
    .await
    .map_err(|e| anyhow::format_err!("Failed to create websocket: {e}"))?;
    let web3s = Web3::new(websocket);

    let root_address = env::var("ROOT_ADDRESS")
        .map_err(|e| anyhow::format_err!("Failed to get env ROOT_ADDRESS: {e}"))?;
    tracing::info!("Root address: {root_address}");
    let elock_address_str = env::var("ETH_CONTRACT_ADDRESS")
        .map_err(|e| anyhow::format_err!("Failed to get env ETH_CONTRACT_ADDRESS: {e}"))?;
    tracing::info!("ELock address: {elock_address_str}");

    let first_block = get_last_gosh_block_id(&elock_address_str, &web3s)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get last GOSH block from ELock: {e}"))?;
    let first_seq_no = get_master_block_seq_no(&context, &first_block)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get seq no for block from ETH: {e}"))?;

    let current_master_block = get_latest_master_block(&context)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get latest GOSH block: {e}"))?;

    tracing::info!(
        "master blocks seq no range: {first_seq_no} - {}",
        current_master_block.seq_no
    );

    let burns = find_burns(
        &context,
        &root_address,
        first_seq_no,
        current_master_block.seq_no,
    )
    .await?;
    tracing::info!("burns: {burns:?}");

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
