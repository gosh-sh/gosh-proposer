use crate::eth::proposal::{check_proposal, create_proposal, get_proposals, vote_for_withdrawal};
use common::gosh::helper::create_client;
use std::env;
use std::str::FromStr;
use serde_json::json;
use web3::contract::Contract;
use web3::signing::SecretKey;
use web3::transports::WebSocket;
use web3::types::{Address, BlockId, BlockNumber};
use web3::Web3;
use common::eth::read_block;
use crate::gosh::block::get_latest_master_block;

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
        match check_proposal(&context, &root_address, &proposal).await {
            Ok(()) => {
                vote_for_withdrawal(proposal.proposal_key, &elock_contract, &key).await?;
            }
            Err(e) => {
                tracing::info!("Proposal check failed for: {proposal:?} {e}");
            }
        }
    }
    Ok(())
}

pub async fn get_last_blocks() -> anyhow::Result<()> {
    let context = create_client()?;

    let websocket = WebSocket::new(&env::var("ETH_NETWORK")?).await?;
    let web3s = Web3::new(websocket);

    let last_gosh_block = get_latest_master_block(
        &context
    ).await?;

    let last_eth_block = read_block(
        &web3s,
        BlockId::Number(BlockNumber::Latest),
    ).await?;

    println!("{}",
        serde_json::to_string_pretty(
            &json!({
                "gosh": last_gosh_block,
                "eth": {
                    "hash": last_eth_block.hash,
                    "number": last_eth_block.number
                }
            })
        )?
    );
    Ok(())
}