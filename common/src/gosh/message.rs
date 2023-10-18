use crate::gosh::helper::EverClient;
use crate::helper::abi::TOKEN_WALLET_ABI;
use crate::helper::deserialize_uint;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use ton_client::abi::{decode_message_body, Abi, ParamsOfDecodeMessageBody};
use ton_client::net::ParamsOfQuery;

pub struct Message {
    pub id: String,
    pub body: String,
    pub tx_id: String,
    pub block_id: String,
    pub lt: u128,
}

#[derive(Deserialize, Debug)]
struct InMessage {
    pub id: String,
    pub body: Option<String>,
    pub msg_type: u8,
}

#[derive(Deserialize, Debug)]
struct Node {
    #[serde(rename = "in_message")]
    message: InMessage,
    aborted: bool,
    lt: String,
    block_id: String,
    id: String,
}

#[derive(Deserialize, Debug)]
struct WrappedNode {
    node: Node,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    has_next_page: bool,
    end_cursor: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Messages {
    edges: Vec<WrappedNode>,
    page_info: PageInfo,
}

#[derive(Deserialize, Debug)]
struct AcceptArguments {
    #[serde(rename = "_value")]
    #[serde(deserialize_with = "deserialize_uint")]
    value: u128,
    #[serde(rename = "answer_addr")]
    _answer_addr: String,
    #[serde(deserialize_with = "deserialize_uint")]
    #[serde(rename = "keep_evers")]
    _keep_evers: u128,
    #[serde(rename = "notify_payload")]
    _notify_payload: Option<String>,
}

pub async fn query_messages(
    context: &EverClient,
    root_address: &str,
    start_seq_no: u128,
    end_seq_no: u128,
) -> anyhow::Result<Vec<Message>> {
    tracing::info!("query transactions to receiver, address={root_address}");
    // Prepare query request
    let query = r#"query($addr: String!, $start: Int, $end: Int, $after: String){
      blockchain {
        account(address: $addr) {
          transactions(
            allow_latest_inconsistent_data: true,
            after: $after,
            master_seq_no_range: {
             start: $start,
             end: $end
           }) {
            edges {
              node {
                in_message {
                  id body msg_type
                }
                aborted
                lt(format: DEC)
                block_id
                id
              }
            }
            pageInfo { hasNextPage endCursor }
          }
        }
      }
    }"#
    .to_string();

    // Init query variables
    let mut after = "".to_string();
    let dst_address = root_address.to_string();
    let mut result_messages = vec![];

    // Start a loop to query all messages in chunks
    loop {
        let result = ton_client::net::query(
            Arc::clone(context),
            ParamsOfQuery {
                query: query.clone(),
                variables: Some(json!({
                    "addr": dst_address.clone(),
                    "start": start_seq_no,
                    "end": end_seq_no,
                    "after": after,
                })),
            },
        )
        .await
        .map(|r| r.result)
        .map_err(|e| anyhow::format_err!("Failed to query data: {e}"))?;

        // Decode query results
        let nodes = &result["data"]["blockchain"]["account"]["transactions"];
        let nodes: Messages = serde_json::from_value(nodes.clone())
            .map_err(|e| anyhow::format_err!("Failed to deserialize query result: {e}"))?;

        // Update start of the queried chunk
        after = nodes.page_info.end_cursor;

        // Decode messages
        for node in nodes.edges {
            let msg = node.node.message;
            if msg.body.is_some() && msg.msg_type == 0 && !node.node.aborted {
                let id = msg.id.trim_start_matches("message/").to_string();
                let tx_id = node.node.id.trim_start_matches("transaction/").to_string();
                let lt =
                    node.node.lt.parse::<u128>().map_err(|e| {
                        anyhow::format_err!("Failed to convert block lt to u128: {e}")
                    })?;
                let message = Message {
                    body: msg.body.unwrap(),
                    id,
                    block_id: node.node.block_id,
                    lt,
                    tx_id,
                };

                result_messages.push(message);
            }
        }

        // Break the loop if there is no next page
        if !nodes.page_info.has_next_page {
            break;
        }
    }
    tracing::info!("Found {} messages to root contract", result_messages.len());
    Ok(result_messages)
}

pub async fn get_token_wallet_total_mint(
    gosh_context: &EverClient,
    wallet_address: &str,
) -> anyhow::Result<u128> {
    tracing::info!("query token transfers to wallet, address={wallet_address}");
    let abi = Abi::Json(TOKEN_WALLET_ABI.to_string());
    let wallet_function_name = "acceptMint";

    let query = r#"query($addr: String!, $after: String){
      blockchain {
        account(address: $addr) {
          transactions(
            allow_latest_inconsistent_data: true,
            after: $after,
           ) {
            edges {
              node {
                in_message {
                  id body msg_type
                }
                aborted
                lt(format: DEC)
                block_id
                id
              }
            }
            pageInfo { hasNextPage endCursor }
          }
        }
      }
    }"#
    .to_string();

    let mut after = "".to_string();
    let dst_address = wallet_address.to_string();
    let mut total_value = 0;
    loop {
        let result = ton_client::net::query(
            Arc::clone(gosh_context),
            ParamsOfQuery {
                query: query.clone(),
                variables: Some(json!({
                    "addr": dst_address.clone(),
                    "after": after,
                })),
            },
        )
        .await
        .map(|r| r.result)
        .map_err(|e| anyhow::format_err!("Failed to query data: {e}"))?;
        let nodes = &result["data"]["blockchain"]["account"]["transactions"];
        let nodes: Messages = serde_json::from_value(nodes.clone())
            .map_err(|e| anyhow::format_err!("Failed to deserialize query result: {e}"))?;

        after = nodes.page_info.end_cursor;
        for node in nodes.edges {
            let msg = node.node.message;
            if msg.body.is_some() && msg.msg_type == 0 && !node.node.aborted {
                let decode_params = ParamsOfDecodeMessageBody {
                    abi: abi.clone(),
                    body: msg.body.unwrap(),
                    is_internal: true,
                    allow_partial: false,
                    function_name: None,
                    data_layout: None,
                };
                let decode_result =
                    decode_message_body(Arc::clone(gosh_context), decode_params).await;
                if let Ok(decode_result) = decode_result {
                    if decode_result.name != wallet_function_name {
                        continue;
                    }
                    let args: AcceptArguments =
                        serde_json::from_value(decode_result.value.unwrap()).map_err(|e| {
                            anyhow::format_err!("Failed to serialize burn arguments: {e}")
                        })?;
                    tracing::info!("Found accept mint: {args:?}");
                    total_value += args.value;
                } else {
                    tracing::info!("Failed to decode message, skip it. ID={}", msg.id);
                }
            }
        }

        if !nodes.page_info.has_next_page {
            break;
        }
    }
    tracing::info!("Total value to the wallet {wallet_address}: {total_value}");
    Ok(total_value)
}
