use rlp::RlpStream;
use sha3::{Digest, Keccak256};
use web3::types::{Block, H256, U256};
use std::str::FromStr;
use tracing_subscriber::fmt::format::Full;
use crate::eth::block::FullBlock;

pub fn serialize_block(block: FullBlock<H256>) -> anyhow::Result<Vec<u8>> {
    tracing::info!("serialize block: {:?}", block);
    let list_len = match block.base_fee_per_gas {
        Some(_) => {
            match block.withdrawals_root {
                Some(_) => 17,
                None => 16,
            }
        }
        None => 15
    };
    let mut rlp_stream = RlpStream::new_list(list_len);
    rlp_stream.append(&block.parent_hash);
    rlp_stream.append(&block.uncles_hash);
    rlp_stream.append(&block.author);
    rlp_stream.append(&block.state_root);
    rlp_stream.append(&block.transactions_root);
    rlp_stream.append(&block.receipts_root);
    rlp_stream.append(&block.logs_bloom.unwrap());
    rlp_stream.append(&block.difficulty);
    rlp_stream.append(&block.number.unwrap());
    rlp_stream.append(&block.gas_limit);
    rlp_stream.append(&block.gas_used);
    rlp_stream.append(&block.timestamp);
    rlp_stream.append(&block.extra_data.0);
    rlp_stream.append(&block.mix_hash.unwrap());
    rlp_stream.append(&block.nonce.unwrap());
    if block.base_fee_per_gas.is_some() {
        rlp_stream.append(&block.base_fee_per_gas.unwrap());
    }
    if block.withdrawals_root.is_some() {
        rlp_stream.append(&block.withdrawals_root.unwrap());
    }
    let out = rlp_stream.out().to_vec();

    let out_str = out.iter().fold(String::new(), |acc, el| format!("{acc}{:02x}", el));
    tracing::info!("encode input: {out_str}");

    let mut hasher = Keccak256::new();
    hasher.update(&out);
    let hash = hasher.finalize();
    let hash_string = hash
        .iter()
        .fold(String::new(), |acc, el| format!("{acc}{:0x}", el));
    tracing::info!(
        "Calculated block hash: {hash_string}.\nOriginal: {:?}",
        block.hash
    );
    assert_eq!(block.hash.unwrap().0.to_vec(), hash.to_vec());

    Ok(out)
}
