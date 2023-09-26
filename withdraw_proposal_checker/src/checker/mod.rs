use crate::eth::proposal::{create_proposal, get_proposals, vote_for_withdrawal};
use common::gosh::helper::create_client;
use std::env;
use std::str::FromStr;
use web3::contract::Contract;
use web3::signing::SecretKey;
use web3::transports::WebSocket;
use web3::types::Address;
use web3::Web3;

const ELOCK_ABI_PATH: &str = "resources/elock.abi.json";

pub async fn check_proposals_and_accept() -> anyhow::Result<()> {
    let context = create_client()?;
    let root_address = env::var("ROOT_ADDRESS")?;
    tracing::info!("Root address: {root_address}");

    let websocket = WebSocket::new(&env::var("ETH_NETWORK")?).await?;
    let web3s = Web3::new(websocket);

    let elock_address_str = env::var("ETH_CONTRACT_ADDRESS")?;
    tracing::info!("elock address: {elock_address_str}");
    let abi_file = std::fs::File::open(ELOCK_ABI_PATH)?;
    let elock_abi = web3::ethabi::Contract::load(abi_file)
        .map_err(|e| anyhow::format_err!("Failed to load elock abi: {e}"))?;
    let elock_address = Address::from_str(&elock_address_str)?;
    let elock_contract = Contract::new(web3s.eth(), elock_address, elock_abi);
    let key = SecretKey::from_str(&env::var("ETH_PRIVATE_KEY")?)
        .map_err(|e| anyhow::format_err!("Failed to load private key: {e}"))?;

    create_proposal(
        &context,
        &web3s,
        &elock_address_str,
        &root_address,
        &elock_contract,
        &key,
    )
    .await?;
    let current_proposals = get_proposals(&elock_contract).await?;
    for proposal in current_proposals {
        // TODO: add check of proposal data

        vote_for_withdrawal(proposal.proposal_key, &elock_contract, &key).await?;
    }
    Ok(())
}
