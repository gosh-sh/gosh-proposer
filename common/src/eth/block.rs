use serde::{Deserialize, Deserializer, Serialize};
use web3::helpers as w3h;
use web3::transports::WebSocket;
use web3::types::{BlockId, Bytes, H160, H2048, H256, H64, U256, U64};
use web3::{Transport, Web3};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct FullBlock<TX> {
    /// Hash of the block
    pub hash: Option<H256>,
    /// Hash of the parent
    #[serde(rename = "parentHash")]
    pub parent_hash: H256,
    /// Hash of the uncles
    #[serde(rename = "sha3Uncles")]
    #[cfg_attr(feature = "allow-missing-fields", serde(default))]
    pub uncles_hash: H256,
    /// Miner/author's address.
    #[serde(rename = "miner", default, deserialize_with = "null_to_default")]
    pub author: H160,
    /// State root hash
    #[serde(rename = "stateRoot")]
    pub state_root: H256,
    /// Transactions root hash
    #[serde(rename = "transactionsRoot")]
    pub transactions_root: H256,
    /// Transactions receipts root hash
    #[serde(rename = "receiptsRoot")]
    pub receipts_root: H256,
    /// Block number. None if pending.
    pub number: Option<U64>,
    /// Gas Used
    #[serde(rename = "gasUsed")]
    pub gas_used: U256,
    /// Gas Limit
    #[serde(rename = "gasLimit")]
    #[cfg_attr(feature = "allow-missing-fields", serde(default))]
    pub gas_limit: U256,
    /// Base fee per unit of gas (if past London)
    #[serde(rename = "baseFeePerGas", skip_serializing_if = "Option::is_none")]
    pub base_fee_per_gas: Option<U256>,
    /// Extra data
    #[serde(rename = "extraData")]
    pub extra_data: Bytes,
    /// Logs bloom
    #[serde(rename = "logsBloom")]
    pub logs_bloom: Option<H2048>,
    /// Timestamp
    pub timestamp: U256,
    /// Difficulty
    #[cfg_attr(feature = "allow-missing-fields", serde(default))]
    pub difficulty: U256,
    /// Total difficulty
    #[serde(rename = "totalDifficulty")]
    pub total_difficulty: Option<U256>,
    /// Seal fields
    #[serde(default, rename = "sealFields")]
    pub seal_fields: Vec<Bytes>,
    /// Uncles' hashes
    #[cfg_attr(feature = "allow-missing-fields", serde(default))]
    pub uncles: Vec<H256>,
    /// Transactions
    pub transactions: Vec<TX>,
    /// Size in bytes
    pub size: Option<U256>,
    /// Mix Hash
    #[serde(rename = "mixHash")]
    pub mix_hash: Option<H256>,
    /// Nonce
    pub nonce: Option<H64>,
    /// Base fee per unit of gas (if past London)
    #[serde(rename = "withdrawalsRoot", skip_serializing_if = "Option::is_none")]
    pub withdrawals_root: Option<H256>,
}

fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let option = Option::deserialize(deserializer)?;
    Ok(option.unwrap_or_default())
}

// Read Ethereum block with specified block id
pub async fn read_block(
    web3s: &Web3<WebSocket>,
    block_id: BlockId,
) -> anyhow::Result<FullBlock<H256>> {
    tracing::info!("Reading block: {block_id:?}");
    let include_txs = w3h::serialize(&false);
    let block = match block_id {
        BlockId::Hash(hash) => {
            let hash = w3h::serialize(&hash);
            web3s
                .transport()
                .execute("eth_getBlockByHash", vec![hash, include_txs])
        }
        BlockId::Number(num) => {
            let num = w3h::serialize(&num);
            web3s
                .transport()
                .execute("eth_getBlockByNumber", vec![num, include_txs])
        }
    }
    .await
    .map_err(|e| anyhow::format_err!("Failed to query ETH block {block_id:?}: {e}"))?;

    serde_json::from_value(block)
        .map_err(|e| anyhow::format_err!("Failed to serialize ETH block: {e}"))
}

#[cfg(test)]
mod tests {
    use super::super::encoder::serialize_block;
    use super::read_block;
    use std::env;
    use web3::transports::WebSocket;
    use web3::types::{BlockId, BlockNumber, U64};
    use web3::Web3;

    #[tokio::test]
    pub async fn test_hash() -> anyhow::Result<()> {
        dotenv::dotenv().ok();
        let block_id = BlockId::Number(BlockNumber::Number(
            U64::from_str_radix("400000", 10).unwrap(),
        ));
        let websocket = WebSocket::new(
            &env::var("ETH_NETWORK")
                .map_err(|e| anyhow::format_err!("Failed to get env ETH_NETWORK: {e}"))?,
        )
        .await
        .map_err(|e| anyhow::format_err!("Failed to create websocket: {e}"))?;
        let web3s = Web3::new(websocket);
        let block = read_block(&web3s, block_id).await?;
        serialize_block(&block)?;
        Ok(())
    }
}
