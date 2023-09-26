use crate::gosh::monitor::query_messages;
use common::gosh::helper::EverClient;
use serde::Deserialize;
use std::sync::Arc;
use ton_client::abi::{decode_message_body, Abi, ParamsOfDecodeMessageBody};

const ROOT_ABI_PATH: &str = "contracts/l2/RootTokenContract.abi";
const ROOT_FUNCTION_NAME: &str = "burn_tokens";

#[derive(Debug)]
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
    start_lt: &str,
) -> anyhow::Result<(Vec<Burn>, String)> {
    let messages = query_messages(context, root_address, start_lt).await?;

    let abi_json = std::fs::read_to_string(ROOT_ABI_PATH)?;
    let abi = Abi::Json(abi_json);

    let mut res = vec![];
    let mut last_trans_lt = 0_u128;
    let mut last_block_id = "".to_string();
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
            if decode_result.name != ROOT_FUNCTION_NAME {
                continue;
            }
            if message.lt > last_trans_lt {
                last_trans_lt = message.lt;
                last_block_id = message.block_id;
            }
            let args: BurnArguments = serde_json::from_value(decode_result.value.unwrap())?;
            res.push(Burn {
                dest: args.to,
                value: args.tokens.parse::<u128>()?,
                tx_id: message.tx_id,
            })
        } else {
            tracing::info!("Failed to decode message, skip it. ID={}", message.id);
        }
    }
    Ok((res, last_block_id))
}
