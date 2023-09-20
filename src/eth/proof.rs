use rlp::RlpStream;
use sha3::{Digest, Keccak256};
use web3::types::{Block, H256};

pub fn serialize_block(block: Block<H256>) -> anyhow::Result<Vec<u8>> {
    tracing::info!("serialize block: {:?}", block);
    let mut rlp_stream = RlpStream::new_list(15);
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
    let out = rlp_stream.out().to_vec();

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
