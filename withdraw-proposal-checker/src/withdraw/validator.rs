use crate::withdraw::proposal::{
    check_proposal, create_proposal, get_proposals, vote_for_withdrawal,
};
use common::elock::get_elock_address;
use common::eth::create_web3_socket;
use common::gosh::helper::create_client;
use common::helper::abi::ELOCK_ABI;

use ethereum_types::BigEndianHash;
use sha3::{Digest, Keccak256};
use std::env;
use std::str::FromStr;
use web3::contract::Contract;
use web3::signing::SecretKey;
use web3::transports::WebSocket;
use web3::types::{Address, H256, U256};
use web3::Web3;

const VOTE_FOR_PROPOSAL_STORAGE_ID: &str =
    "000000000000000000000000000000000000000000000000000000000000000D";

fn get_secret() -> anyhow::Result<SecretKey> {
    let key_path = env::var("ETH_PRIVATE_KEY_PATH")
        .map_err(|e| anyhow::format_err!("Failed to get env ETH_PRIVATE_KEY_PATH: {e}"))?;
    SecretKey::from_str(
        std::fs::read_to_string(key_path)
            .map_err(|e| anyhow::format_err!("Failed to read ETH_PRIVATE_KEY_PATH: {e}"))?
            .trim(),
    )
    .map_err(|e| anyhow::format_err!("Failed to load private key: {e}"))
}

pub async fn create_new_proposal() -> anyhow::Result<()> {
    // Create client for GOSH
    let context = create_client()?;

    // Create client for ETH
    let web3s = create_web3_socket().await?;

    // Load ELock contract
    let elock_address = get_elock_address()?;
    let elock_abi = web3::ethabi::Contract::load(ELOCK_ABI.as_bytes())
        .map_err(|e| anyhow::format_err!("Failed to load elock abi: {e}"))?;
    let elock_contract = Contract::new(web3s.eth(), elock_address, elock_abi);

    // Load validator ETH key
    let key = get_secret()?;

    create_proposal(&context, &web3s, elock_address, &elock_contract, &key).await?;
    Ok(())
}

async fn has_voted(
    web3s: &Web3<WebSocket>,
    elock_address: Address,
    proposal_key: &H256,
    validator_address: &H256,
) -> anyhow::Result<bool> {
    tracing::info!("Check validators vote for proposal");
    // keccak256(
    //     uint256(VALIDATOR_ADDR) . keccak256(uint256(PROPOSAL_KEY) . uint256(0xd))
    // )
    let mut hasher = Keccak256::new();
    hasher.update(proposal_key.as_bytes());
    let index = H256::from_str(VOTE_FOR_PROPOSAL_STORAGE_ID)?;
    hasher.update(index.as_bytes());
    let hash = hasher.finalize();

    let mut hasher = Keccak256::new();
    hasher.update(validator_address.as_bytes());
    hasher.update(hash);
    let storage_key = hasher.finalize();
    let idx = U256::from_big_endian(storage_key.as_ref());

    let res = web3s.eth().storage(elock_address, idx, None).await?;
    tracing::info!("Check validators vote for proposal result: {res}");

    Ok(!res.is_zero())
}

pub async fn check_proposals_and_accept() -> anyhow::Result<()> {
    // Create client for GOSH
    let context = create_client()?;

    // Create client for ETH
    let web3s = create_web3_socket().await?;

    // Load ELock contract
    let elock_address_str = env::var("ETH_CONTRACT_ADDRESS")
        .map_err(|e| anyhow::format_err!("Failed to get env ETH_CONTRACT_ADDRESS: {e}"))?;
    tracing::info!("elock address: {elock_address_str}");
    let elock_abi = web3::ethabi::Contract::load(ELOCK_ABI.as_bytes())
        .map_err(|e| anyhow::format_err!("Failed to load elock abi: {e}"))?;
    let elock_address = Address::from_str(&elock_address_str)
        .map_err(|e| anyhow::format_err!("Failed to convert ETH address: {e}"))?;
    let elock_contract = Contract::new(web3s.eth(), elock_address, elock_abi);

    // Load validator's ETH key
    let key = get_secret()?;

    // Load Validators wallet address
    let validator_address_str = env::var("ETH_VALIDATOR_CONTRACT_ADDRESS").map_err(|e| {
        anyhow::format_err!("Failed to get env ETH_VALIDATOR_CONTRACT_ADDRESS: {e}")
    })?;

    // Format address
    let validator_address = Address::from_str(&validator_address_str)
        .map_err(|e| anyhow::format_err!("Failed to convert ETH address: {e}"))?;
    let validator_address_bytes = validator_address.to_fixed_bytes();
    let mut validator_address_padded = [0_u8; 32];
    validator_address_padded[12..].copy_from_slice(&validator_address_bytes[..]);
    let validator_address = H256::from(validator_address_padded);
    tracing::info!("validator_address: {validator_address:?}");

    // Get list of proposals from ELock
    let current_proposals = get_proposals(&elock_contract).await?;
    for proposal in current_proposals {
        match has_voted(
            &web3s,
            elock_address,
            &H256::from_uint(&proposal.proposal_key),
            &validator_address,
        )
        .await
        {
            Ok(val) => {
                tracing::info!("Vote query result: {val}");
                if val {
                    continue;
                }
            }
            Err(e) => {
                tracing::info!("Failed to check vote: {e}");
                continue;
            }
        };
        match check_proposal(&context, &proposal).await {
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
