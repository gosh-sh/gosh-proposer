use std::sync::Arc;
use serde::Deserialize;
use serde_json::json;
use ton_client::net::ParamsOfQuery;
use common::gosh::helper::EverClient;

// query {
//   blocks(
//     filter: {
//       id: {
//         eq: "edbb5d6223715f78e9a2c6fd953533aa48cf1df5c21a9704fc7ffa4e60647214"
//       }
//     }
//   ) {
//     id end_lt(format:DEC)
//  }
// }

// {
//   "data": {
//     "blocks": [
//       {
//         "id": "edbb5d6223715f78e9a2c6fd953533aa48cf1df5c21a9704fc7ffa4e60647214",
//         "end_lt": "384222000001"
//       }
//     ]
//   }
// }

#[derive(Deserialize, Debug)]
struct Block {
    id: String,
    end_lt: String,
}

#[derive(Deserialize, Debug)]
struct Blocks {
    blocks: Vec<Block>,
}

pub async fn get_block_lt(
    context: &EverClient,
    block_id: &str,
) -> anyhow::Result<String> {
    tracing::info!("query end block lt for block_id={block_id}");
    let query = r#"query($block_id: String){
        blocks(
            filter: {
              id: {
                eq: $block_id
              }
            }
          ) {
            id end_lt(format:DEC)
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

    let blocks: Blocks = serde_json::from_value(result["data"].clone())
        .map_err(|e| anyhow::format_err!("Failed to deserialize query result: {e}"))?;

    if blocks.blocks.len() != 1 {
        anyhow::bail!("Failed to find block with specified id: {}", block_id);
    }

    Ok(blocks.blocks[0].end_lt.clone())
}