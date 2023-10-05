use std::env;
use std::str::FromStr;
use web3::contract::tokens::Tokenize;
use web3::contract::{Contract, Options};
use web3::signing::SecretKey;
use web3::transports::WebSocket;
use web3::types::{U256, U64};

const ETH_CALL_GAS_LIMIT: u128 = 1000000;
const ETH_TRANSACTION_TYPE: u64 = 2;
const DEFAULT_CONFIRMATIONS_CNT: usize = 1;

fn get_options() -> Options {
    Options {
        transaction_type: Some(U64::from(ETH_TRANSACTION_TYPE)),
        gas: Some(U256::from(ETH_CALL_GAS_LIMIT)),
        ..Default::default()
    }
}

pub async fn call_function<T: Tokenize>(
    elock_contract: &Contract<WebSocket>,
    key: &SecretKey,
    function: &str,
    params: T,
) -> anyhow::Result<()> {
    tracing::info!("Call ETH contract function {function}");

    let options = get_options();
    let confirmation_cnt = env::var("ETH_CONFIRMATIONS_CNT")
        .ok()
        .and_then(|s| usize::from_str(&s).ok())
        .unwrap_or(DEFAULT_CONFIRMATIONS_CNT);
    let res = elock_contract
        .signed_call_with_confirmations(function, params, options, confirmation_cnt, key)
        .await
        .map_err(|e| anyhow::format_err!("Failed to call ELock function {function}: {e}"))?;
    tracing::info!("ETH call result: {}", web3::helpers::to_string(&res));
    Ok(())
}
