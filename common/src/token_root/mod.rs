pub mod eth;
mod gosh;

use crate::helper::deserialize_uint;
pub use gosh::{
    get_root_address, get_root_owner_address, get_root_owner_balance, get_wallet_balance,is_root_active,deploy_root
};
use serde::{Deserialize, Deserializer, Serialize, de::Error};
use web3::types::Address;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct RootData {
    pub name: String,
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_uint")]
    pub decimals: u8,
    #[serde(rename = "ethroot")]
    #[serde(deserialize_with = "deserialize_address")]
    pub eth_root: Address,
}

pub fn deserialize_address<'de, D>(deserializer: D) -> Result<Address, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    assert!(s.starts_with("0x"));
    assert_eq!(s.len(), 66);
    Address::from_str(&s[26..66].to_string()).map_err(D::Error::custom)
}