use std::env;
use crate::gosh::monitor::query_messages;
use common::gosh::helper::EverClient;
use serde::Deserialize;
use std::sync::Arc;
use ton_client::abi::{decode_message_body, Abi, ParamsOfDecodeMessageBody};

const ROOT_ABI_PATH: &str = "contracts/l2/RootTokenContract.abi";

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
    let messages = query_messages(context, root_address, start_seq_no, end_seq_no).await?;

    let abi_json = std::fs::read_to_string(ROOT_ABI_PATH)?;
    let abi = Abi::Json(abi_json);
    let root_function_name = env::var("ROOT_FUNCTION_NAME")?;

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
            let args: BurnArguments = serde_json::from_value(decode_result.value.unwrap())?;
            let trimmed_to = args.to[26..].to_string();
            let dest = format!("0x{}", trimmed_to);
            res.push(Burn {
                dest,
                value: args.tokens.parse::<u128>()?,
                tx_id: message.tx_id,
            })
        } else {
            tracing::info!("Failed to decode message, skip it. ID={}", message.id);
        }
    }
    Ok(res)
}
