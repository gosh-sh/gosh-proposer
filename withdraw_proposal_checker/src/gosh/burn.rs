use std::sync::Arc;
use serde::Deserialize;
use ton_client::abi::{Abi, decode_message_body, ParamsOfDecodeMessageBody};
use common::gosh::helper::EverClient;
use crate::gosh::monitor::query_messages;

const ROOT_ABI_PATH: &str = "contracts/l2/RootTokenContract.abi";
const ROOT_FUNCTION_NAME: &str = "burn_tokens";

#[derive(Debug)]
pub struct Burn {
    pub dest: String,
    pub value: u128,
    pub msg_id: String,
}

#[derive(Deserialize)]
struct BurnArguments {
    _answer_id: String,
    #[serde(rename = "pubkey")]
    _pubkey: String,
    #[serde(rename = "owner")]
    _owner: String,
    tokens: String,
    to: String,
}

pub async fn find_burns(
    context: &EverClient,
    root_address: &str,
) -> anyhow::Result<Vec<Burn>> {
    let messages = query_messages(
        context,
        root_address,
    ).await?;

    let abi_json = std::fs::read_to_string(ROOT_ABI_PATH)?;
    let abi = Abi::Json(abi_json);

    let mut res= vec![];
    for message in messages {
        let decode_params = ParamsOfDecodeMessageBody {
            abi: abi.clone(),
            body: message.body,
            is_internal: true,
            allow_partial: true,
            function_name: None,
            data_layout: None,
        };
        let decode_result = decode_message_body(
            Arc::clone(context),
            decode_params,
        ).await?;
        if decode_result.name != ROOT_FUNCTION_NAME {
            continue;
        }
        let args: BurnArguments = serde_json::from_value(decode_result.value.unwrap())?;
        res.push(Burn {
            dest: args.to,
            value: u128::from_str_radix(&args.tokens, 10)?,
            msg_id: message.id,
        })
    }
    Ok(res)
}