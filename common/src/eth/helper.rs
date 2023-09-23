use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use web3::types::U256;

const IDENTIFIERS_PATH: &str = "../resources/identifiers.json";

pub fn wei_to_eth(wei_val: U256) -> f64 {
    let res = wei_val.as_u128() as f64;
    res / 1_000_000_000_000_000_000.0
}

pub fn get_signatures_table() -> anyhow::Result<BTreeMap<String, Vec<String>>> {
    let file = File::open(IDENTIFIERS_PATH)
        .map_err(|e| anyhow::format_err!("Failed to open file {}: {}", IDENTIFIERS_PATH, e))?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader)
        .map_err(|e| anyhow::format_err!("Failed to decode identifiers map {}", e))
}
