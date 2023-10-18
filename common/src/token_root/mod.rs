pub mod eth;
mod gosh;

use crate::helper::deserialize_uint;
pub use gosh::{
    get_root_address, get_root_owner_address, get_root_owner_balance, get_wallet_balance,
};
use serde::{Deserialize, Serialize};
use web3::types::Address;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct RootData {
    pub name: String,
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_uint")]
    pub decimals: u8,
    #[serde(rename = "ethroot")]
    pub eth_root: Address,
}
