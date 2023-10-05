use crate::gosh::helper::EverClient;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use ton_client::net::ParamsOfQuery;

#[derive(Deserialize, Debug, Serialize)]
pub struct MasterBlock {
    pub seq_no: u128,
    #[serde(rename = "id")]
    pub block_id: String,
}

#[derive(Deserialize, Debug)]
struct SeqNo {
    seq_no: u128,
    workchain_id: i8,
}

pub async fn get_master_block_seq_no(context: &EverClient, block_id: &str) -> anyhow::Result<u128> {
    tracing::info!("query seq no for block_id={block_id}");
    let query = r#"query($block_id: String!){
        blockchain {
            block(
                hash: $block_id
            ) {
                seq_no workchain_id
            }
        }
    }"#
    .to_string();

    let block_id = block_id.to_string();
    let result = ton_client::net::query(
        Arc::clone(context),
        ParamsOfQuery {
            query: query.clone(),
            variables: Some(json!({
                "block_id": block_id
            })),
        },
    )
    .await
    .map(|r| r.result)
    .map_err(|e| anyhow::format_err!("Failed to query data: {e}"))?;

    tracing::info!("query result: {result}");

    let seq_no: SeqNo = serde_json::from_value(result["data"]["blockchain"]["block"].clone())
        .map_err(|e| anyhow::format_err!("Failed to deserialize query result: {e}"))?;
    tracing::info!("queried seq_no: {seq_no:?}");
    if seq_no.workchain_id != -1 {
        anyhow::bail!("Specified block is not a masterchain block");
    }
    Ok(seq_no.seq_no)
}

pub async fn get_latest_master_block(context: &EverClient) -> anyhow::Result<MasterBlock> {
    tracing::info!("query latest master block seq no");
    let query = r#"query {
        blockchain {
            blocks( allow_latest_inconsistent_data: true, last: 1, workchain: -1 ) {
                edges { node { seq_no id }  }
            }
        }
    }"#
    .to_string();

    let result = ton_client::net::query(
        Arc::clone(context),
        ParamsOfQuery {
            query: query.clone(),
            variables: None,
        },
    )
    .await
    .map(|r| r.result)
    .map_err(|e| anyhow::format_err!("Failed to query data: {e}"))?;

    tracing::info!("query result: {result}");

    let mut master_block: MasterBlock =
        serde_json::from_value(result["data"]["blockchain"]["blocks"]["edges"][0]["node"].clone())
            .map_err(|e| anyhow::format_err!("Failed to deserialize query result: {e}"))?;

    master_block.block_id = master_block
        .block_id
        .trim_start_matches("block/")
        .to_string();
    tracing::info!("queried seq_no: {master_block:?}");
    Ok(master_block)
}
