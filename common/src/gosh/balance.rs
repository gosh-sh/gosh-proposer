use std::sync::Arc;
use serde_json::json;
use ton_client::net::ParamsOfQuery;
use crate::gosh::helper::EverClient;

pub async fn query_balance(
    context: &EverClient,
    address: &str,
) -> anyhow::Result<u128> {
    tracing::info!("query account balance, address={address}");
    // Prepare query request
    let query = r#"query($addr: String!){
      blockchain {
        account(address: $addr) {
          info {
		    balance(format: DEC)
          }
        }
      }
    }"#
        .to_string();

    // Init query variables
    let dst_address = address.to_string();


    let result = ton_client::net::query(
        Arc::clone(context),
        ParamsOfQuery {
            query: query.clone(),
            variables: Some(json!({
                "addr": dst_address.clone(),
            })),
        },
    )
        .await
        .map(|r| r.result)
        .map_err(|e| anyhow::format_err!("Failed to query data: {e}"))?;

    // Decode query results
    let result = u128::from_str_radix(
        &result["data"]["blockchain"]["account"]["info"]["balance"].as_str()
            .ok_or(anyhow::format_err!("Failed to decode account balance"))?,
        10
    )?;

    tracing::info!("{} balance: {}", address, result);
    Ok(result)
}