use std::env;
use web3::contract::{Contract, Options};
use web3::transports::WebSocket;
use web3::types::{Address, U256};
use web3::Web3;
use common::gosh::helper::create_client;
use crate::eth::elock::get_last_gosh_block_id;
use crate::gosh::block::get_block_lt;
use crate::gosh::burn::{Burn, find_burns};
use std::str::FromStr;
use web3::ethabi::Token;
use web3::signing::SecretKey;

const ELOCK_ABI_PATH: &str = "resources/elock.abi.json";

pub async fn create_proposal() -> anyhow::Result<()> {
    let context = create_client()?;
    let elock_address = env::var("ETH_CONTRACT_ADDRESS")?;
    let websocket = WebSocket::new(&env::var("ETH_NETWORK")?).await?;
    let web3s = Web3::new(websocket);

    let first_block = get_last_gosh_block_id(&elock_address, &web3s).await?;
    let block_lt = get_block_lt(&context, &first_block).await?;

    let root_address = env::var("ROOT_ADDRESS")?;
    let (burns, last_block) = find_burns(&context, &root_address, &block_lt).await?;
    tracing::info!("burns: {burns:?}");

    let burns = convert_burns(burns)
        .map_err(|e| anyhow::format_err!("Failed to convert burns: {e}"))?;

    let elock_address = Address::from_str(&elock_address)?;
    let abi_file = std::fs::File::open(ELOCK_ABI_PATH)?;
    let elock_abi = web3::ethabi::Contract::load(abi_file)
        .map_err(|e| anyhow::format_err!("Failed to load elock abi: {e}"))?;
    let elock_contract = Contract::new(web3s.eth(), elock_address, elock_abi);
    let key = SecretKey::from_str(&env::var("ETH_PRIVATE_KEY")?)
        .map_err(|e| anyhow::format_err!("Failed to load private key: {e}"))?;

    // tracing::info!("Start getter call");
    // let res = elock_contract.signed_call(
    //     "getProposalList",
    //     (),
    //     elock_address,
    //     Options::default()
    // ).await?;
    //
    // tracing::info!("getter result: {res}");

    let first_block = Token::Uint(U256::from_str(&first_block)?);
    let last_block = Token::Uint(U256::from_str(&last_block)?);

    tracing::info!("Start call");
    tracing::info!("{first_block} {last_block} {burns:?}");
    let mut options = Options::default();
    options.value = Some(U256::from(1000000000000000_u128));
    let res = elock_contract.signed_call(
        "deposit",
        (first_block),
        options,
        &key,
    ).await?;
    tracing::info!("Call result: {}", web3::helpers::to_string(&res));

    Ok(())
}

fn convert_burns(burns: Vec<Burn>) -> anyhow::Result<Vec<Token>> {
    let mut res = vec![];
    for burn in burns {
        let mut tuple = vec![];
        let trimmed_dest = burn.dest[26..].to_string();
        let to = Token::Address(Address::from_str(&trimmed_dest)?);
        let value = Token::Uint(U256::from(burn.value));
        let tx_is = Token::Uint(U256::from_str(&burn.tx_id)?);
        tuple.push(to);
        tuple.push(value);
        tuple.push(tx_is);
        res.push(Token::Tuple(tuple));
    }
    Ok(res)
}
