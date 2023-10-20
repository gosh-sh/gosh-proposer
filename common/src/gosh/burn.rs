use crate::gosh::helper::EverClient;
use crate::gosh::message::query_messages;
use crate::helper::abi::RECEIVER_ABI;
use serde::Deserialize;

use crate::checker::get_receiver_address;
use crate::token_root::RootData;
use std::sync::Arc;
use ton_client::abi::{decode_message_body, Abi, ParamsOfDecodeMessageBody};

const RECEIVER_FUNCTION_NAME: &str = "burnTokens";

#[derive(Debug, PartialEq)]
pub struct Burn {
    pub dest: String,
    pub value: u128,
    pub tx_id: String,
    pub eth_root: String,
}

#[derive(Deserialize)]
struct BurnArguments {
    root: RootData,
    #[serde(rename = "pubkey")]
    _pubkey: String,
    #[serde(rename = "owner")]
    _owner: Option<String>,
    tokens: String,
    to: String,
}

pub async fn find_burns(
    context: &EverClient,
    start_seq_no: u128,
    end_seq_no: u128,
) -> anyhow::Result<Vec<Burn>> {
    // Get receiver address
    let receiver_address = get_receiver_address(context).await?;

    // Query all messages to receiver
    let messages = query_messages(context, &receiver_address, start_seq_no, end_seq_no)
        .await
        .map_err(|e| anyhow::format_err!("Failed to query messages to ROOT: {e}"))?;

    // Load receiver abi
    let abi = Abi::Json(RECEIVER_ABI.to_string());

    // Decode messages and look for message with burn
    let mut res = vec![];
    for message in messages {
        let body = message.body;
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
            if decode_result.name != RECEIVER_FUNCTION_NAME {
                continue;
            }
            // Decode arguments of burn function call
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
                eth_root: args.root.eth_root.to_string(),
            })
        } else {
            tracing::info!("Failed to decode message, skip it. ID={}", message.id);
        }
    }
    Ok(res)
}
