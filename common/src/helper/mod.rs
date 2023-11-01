use crate::eth::{create_web3_socket, read_block};
use crate::gosh::block::get_latest_master_block;
use crate::gosh::helper::create_client;
use serde::{Deserialize, Deserializer, Serializer};
use serde_json::json;
use web3::types::{BlockId, BlockNumber};

pub mod abi;
pub mod tracing;

pub fn deserialize_uint<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr,
    T: std::default::Default,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.parse::<T>().unwrap_or_default())
}

pub fn serialize_u128<S>(val: &u128, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    let val_str = format!("{val}");
    s.serialize_str(&val_str)
}

pub async fn get_last_blocks() -> anyhow::Result<()> {
    let context = create_client()?;

    let web3s = create_web3_socket().await?;

    let last_gosh_block = get_latest_master_block(&context)
        .await
        .map_err(|e| anyhow::format_err!("Failed latest master block: {e}"))?;

    let last_eth_block = read_block(&web3s, BlockId::Number(BlockNumber::Finalized)).await?;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "gosh": last_gosh_block,
            "eth": {
                "hash": last_eth_block.hash,
                "number": last_eth_block.number
            }
        }))
        .map_err(|e| anyhow::format_err!("Failed to serialize last block: {e}"))?
    );
    Ok(())
}
