use crate::helper::abi::ERC20_ABI;
use crate::token_root::RootData;
use web3::contract::{Contract, Options};
use web3::transports::WebSocket;
use web3::types::Address;
use web3::Web3;

// Wrapped GOSH ETH token data:
pub const GETH_NAME: &str = "geth";
pub const GETH_SYMBOL: &str = "gth";
pub const GETH_DECIMALS: u8 = 18;
pub const GETH_ADDRESS: Address = Address::zero();

pub fn get_geth_root_data() -> RootData {
    RootData {
        name: GETH_NAME.to_string(),
        symbol: GETH_SYMBOL.to_string(),
        decimals: GETH_DECIMALS,
        eth_root: GETH_ADDRESS,
    }
}

pub async fn get_root_data(web3s: &Web3<WebSocket>, address: Address) -> anyhow::Result<RootData> {
    if address.is_zero() {
        return Ok(get_geth_root_data());
    }
    let root_abi = web3::ethabi::Contract::load(ERC20_ABI.as_bytes())?;
    let root_contract = Contract::new(web3s.eth(), address, root_abi);

    let name: String = root_contract
        .query("name", (), None, Options::default(), None)
        .await?;

    let symbol: String = root_contract
        .query("symbol", (), None, Options::default(), None)
        .await?;

    let decimals: u8 = root_contract
        .query("decimals", (), None, Options::default(), None)
        .await?;

    Ok(RootData {
        name,
        symbol,
        decimals,
        eth_root: address,
    })
}
