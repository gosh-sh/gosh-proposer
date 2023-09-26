use crate::eth::block::FullBlock;
use std::env;
use web3::transports::WebSocket;
use web3::types::{Address, BlockId, BlockNumber, Bytes, H256, U256};
use web3::Web3;
use web3::{helpers as w3h, Transport};

pub mod block;
pub mod encoder;
pub mod helper;
pub mod transfer;

#[derive(Debug)]
pub struct StorageProofValue {
    pub value: U256,
    // Array of rlp-serialized MerkleTree-Nodes, starting with the storageHash-Node,
    // following the path of the SHA3 (key) as path.
    pub proof: Vec<Bytes>,
}

#[derive(Debug)]
pub enum StorageProof {
    TotalSupply(StorageProofValue),
    TrxCount(StorageProofValue),
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
    .await?;

    Ok(serde_json::from_value(block)?)
}

pub async fn _get_storage_proof(
    web3s: &Web3<WebSocket>,
    account: Address,
    block_num: Option<BlockNumber>,
) -> anyhow::Result<Vec<StorageProof>> {
    let keys = vec![
        U256::from_str_radix(&env::var("ELOCK_INDEX_TOTAL_SUPPLY")?, 16)?, // totalSupply storage slot
        U256::from_str_radix(&env::var("ELOCK_INDEX_TRX_COUNT")?, 16)?,    // trxCount storage slot
    ];

    let proof = web3s.eth().proof(account, keys, block_num).await?;

    let storage_proofs = match proof {
        Some(proof) => {
            let total_supply_proof = StorageProofValue {
                value: proof.storage_proof[0].value,
                proof: proof.storage_proof[0].proof.clone(),
            };
            let trx_count_proof = StorageProofValue {
                value: proof.storage_proof[1].value,
                proof: proof.storage_proof[1].proof.clone(),
            };
            vec![
                StorageProof::TotalSupply(total_supply_proof),
                StorageProof::TrxCount(trx_count_proof),
            ]
        }
        None => vec![],
    };

    Ok(storage_proofs)
}

#[cfg(test)]
mod tests {
    use super::encoder::serialize_block;
    use super::{get_storage_proof, read_block, StorageProof};
    use crate::helper::tracing::init_default_tracing;
    use std::env;
    use std::matches;
    use std::str::FromStr;
    use web3::transports::WebSocket;
    use web3::types::{Address, BlockId, BlockNumber, U64};
    use web3::Web3;

    #[tokio::test]
    pub async fn test_hash() -> anyhow::Result<()> {
        dotenv::dotenv().ok();
        let block_id = BlockId::Number(BlockNumber::Number(
            U64::from_str_radix("400000", 10).unwrap(),
        ));
        let websocket = WebSocket::new(&env::var("ETH_NETWORK")?).await?;
        let web3s = Web3::new(websocket);
        let block = read_block(&web3s, block_id).await?;
        serialize_block(block)?;
        Ok(())
    }

    #[tokio::test]
    pub async fn ensure_got_storage_proof() -> anyhow::Result<()> {
        dotenv::dotenv().ok();
        init_default_tracing();
        let websocket = WebSocket::new(&env::var("ETH_NETWORK")?).await?;
        let web3s = Web3::new(websocket);

        let block_num = Some(BlockNumber::Latest);
        let account = Address::from_str("0x52410a00621a9bc08f8230a27267957913d961b3")?;
        let storage_proof = get_storage_proof(&web3s, account, block_num).await?;

        assert_eq!(storage_proof.len(), 2);
        assert!(matches!(storage_proof[0], StorageProof::TotalSupply { .. }));
        assert!(matches!(storage_proof[1], StorageProof::TrxCount { .. }));

        Ok(())
    }
}
