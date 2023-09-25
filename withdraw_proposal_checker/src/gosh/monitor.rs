use common::gosh::helper::EverClient;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
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

pub async fn query_messages(
    context: &EverClient,
    root_address: &str,
    start_lt: &str,
) -> anyhow::Result<Vec<Message>> {
    tracing::info!("query messages to root, address={root_address}");
    let query = r#"query($addr: String!, $after: String){
      blockchain {
        account(address: $addr) {
          transactions(after: $after, first: 20) {
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

    let mut after = start_lt.to_string();
    let dst_address = root_address.to_string();
    let mut result_messages = vec![];

    loop {
        let result = ton_client::net::query(
            Arc::clone(context),
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
            if msg.body.is_some() && msg.msg_type == 0 && node.node.aborted == false {
                let id = msg.id.trim_start_matches("message/").to_string();
                let tx_id = node.node.id.trim_start_matches("transaction/").to_string();
                let message = Message {
                    body: msg.body.unwrap(),
                    id,
                    block_id: node.node.block_id,
                    lt: node.node.lt.parse::<u128>()?,
                    tx_id,
                };

                result_messages.push(message);
            }
        }

        if !nodes.page_info.has_next_page {
            break;
        }
    }
    tracing::info!("Found {} messages to root contract", result_messages.len());
    Ok(result_messages)
}
