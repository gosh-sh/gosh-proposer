use crate::helper::deserialize_uint;
use crate::token_root::RootData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransferPatch {
    pub root: RootData,
    pub data: Transfer,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Transfer {
    pub pubkey: String,
    #[serde(deserialize_with = "deserialize_uint")]
    pub value: u128,
    pub hash: String,
}
