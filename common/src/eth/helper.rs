use std::collections::BTreeMap;
use std::io::BufReader;
use web3::types::U256;
use crate::helper::abi::ELOCK_IDS;

pub fn wei_to_eth(wei_val: U256) -> f64 {
    let res = wei_val.as_u128() as f64;
    res / 1_000_000_000_000_000_000.0
}

pub fn get_signatures_table() -> anyhow::Result<BTreeMap<String, Vec<String>>> {
    let reader = BufReader::new(ELOCK_IDS.as_bytes());
    serde_json::from_reader(reader)
        .map_err(|e| anyhow::format_err!("Failed to decode identifiers map {}", e))
}
