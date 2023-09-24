use common::gosh::helper::EverClient;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use ton_client::net::ParamsOfQuery;

#[derive(Deserialize, Debug)]
pub struct InMessage {
    pub id: String,
    pub body: String,
}

#[derive(Deserialize, Debug)]
struct Node {
    #[serde(rename = "node")]
    message: InMessage,
    #[serde(rename = "lt")]
    _lt: String,
    #[serde(rename = "block_id")]
    _block_id: String,
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
    edges: Vec<Node>,
    page_info: PageInfo,
}

pub async fn query_messages(
    context: &EverClient,
    root_address: &str,
) -> anyhow::Result<Vec<InMessage>> {
    let query = r#"query($addr: String!, $before: String){
      blockchain {
        account(address: $addr) {
          transactions_by_lt(after: $after, first: 20) {
            edges {
              node {
                in_message {
                  id body msg_type
                }
                lt(format: DEC)
                block_id
              }
            }
            pageInfo { hasNextPage endCursor }
          }
        }
      }
    }"#
    .to_string();

    let mut after = "".to_string();
    let dst_address = root_address.to_string();
    let mut result_messages = vec![];

    loop {
        let result = ton_client::net::query(
            Arc::clone(context),
            ParamsOfQuery {
                query: query.clone(),
                variables: Some(json!({
                    "addr": dst_address.clone(),
                    "after": after
                })),
            },
        )
        .await
        .map(|r| r.result)
        .map_err(|e| anyhow::format_err!("Failed to query data: {e}"))?;
        let nodes = &result["data"]["blockchain"]["account"]["transactions_by_lt"];
        let nodes: Messages = serde_json::from_value(nodes.clone())
            .map_err(|e| anyhow::format_err!("Failed to deserialize query result: {e}"))?;

        after = nodes.page_info.end_cursor;
        for node in nodes.edges {
            result_messages.push(node.message);
        }

        if !nodes.page_info.has_next_page {
            break;
        }
    }

    Ok(result_messages)
}
