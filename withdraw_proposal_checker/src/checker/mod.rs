use crate::eth::proposal::{check_proposal, create_proposal, get_proposals, vote_for_withdrawal};
use crate::gosh::block::get_latest_master_block;
use common::eth::read_block;
use common::gosh::helper::create_client;
use common::helper::abi::ELOCK_ABI;
use ethereum_types::BigEndianHash;
use serde_json::json;
use sha3::{Digest, Keccak256};
use std::env;
use std::str::FromStr;
use web3::contract::Contract;
use web3::signing::SecretKey;
use web3::transports::WebSocket;
use web3::types::{Address, BlockId, BlockNumber, H256, U256};
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
    let context = create_client()?;

    let root_address = env::var("ROOT_ADDRESS")
        .map_err(|e| anyhow::format_err!("Failed to get env ROOT_ADDRESS: {e}"))?;
    tracing::info!("Root address: {root_address}");

    let websocket = WebSocket::new(
        &env::var("ETH_NETWORK")
            .map_err(|e| anyhow::format_err!("Failed to get env ETH_NETWORK: {e}"))?,
    )
    .await
    .map_err(|e| anyhow::format_err!("Failed to create websocket: {e}"))?;
    let web3s = Web3::new(websocket);

    let elock_address_str = env::var("ETH_CONTRACT_ADDRESS")
        .map_err(|e| anyhow::format_err!("Failed to get env ETH_CONTRACT_ADDRESS: {e}"))?;
    tracing::info!("elock address: {elock_address_str}");
    let elock_abi = web3::ethabi::Contract::load(ELOCK_ABI.as_bytes())
        .map_err(|e| anyhow::format_err!("Failed to load elock abi: {e}"))?;
    let elock_address = Address::from_str(&elock_address_str)
        .map_err(|e| anyhow::format_err!("Failed to convert ETH address: {e}"))?;
    let elock_contract = Contract::new(web3s.eth(), elock_address, elock_abi);

    let key = get_secret()?;

    create_proposal(
        &context,
        &web3s,
        &elock_address_str,
        &root_address,
        &elock_contract,
        &key,
    )
    .await?;
    Ok(())
}

pub async fn has_voted(
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
    let context = create_client()?;

    let root_address = env::var("ROOT_ADDRESS")
        .map_err(|e| anyhow::format_err!("Failed to get env ROOT_ADDRESS: {e}"))?;
    tracing::info!("Root address: {root_address}");

    let websocket = WebSocket::new(
        &env::var("ETH_NETWORK")
            .map_err(|e| anyhow::format_err!("Failed to get env ETH_NETWORK: {e}"))?,
    )
    .await
    .map_err(|e| anyhow::format_err!("Failed to create websocket: {e}"))?;
    let web3s = Web3::new(websocket);

    let elock_address_str = env::var("ETH_CONTRACT_ADDRESS")
        .map_err(|e| anyhow::format_err!("Failed to get env ETH_CONTRACT_ADDRESS: {e}"))?;
    tracing::info!("elock address: {elock_address_str}");
    let elock_abi = web3::ethabi::Contract::load(ELOCK_ABI.as_bytes())
        .map_err(|e| anyhow::format_err!("Failed to load elock abi: {e}"))?;
    let elock_address = Address::from_str(&elock_address_str)
        .map_err(|e| anyhow::format_err!("Failed to convert ETH address: {e}"))?;
    let elock_contract = Contract::new(web3s.eth(), elock_address, elock_abi);

    let key = get_secret()?;

    let validator_address_str = env::var("ETH_VALIDATOR_CONTRACT_ADDRESS").map_err(|e| {
        anyhow::format_err!("Failed to get env ETH_VALIDATOR_CONTRACT_ADDRESS: {e}")
    })?;
    let validator_address = Address::from_str(&validator_address_str)
        .map_err(|e| anyhow::format_err!("Failed to convert ETH address: {e}"))?;
    let validator_address_bytes = validator_address.to_fixed_bytes();
    let mut validator_address_padded = [0_u8; 32];
    for i in 0..20 {
        validator_address_padded[i + 12] = validator_address_bytes[i];
    }

    let validator_address = H256::from(validator_address_padded);
    tracing::info!("validator_address: {validator_address:?}");

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

    let websocket = WebSocket::new(
        &env::var("ETH_NETWORK")
            .map_err(|e| anyhow::format_err!("Failed to get env ETH_NETWORK: {e}"))?,
    )
    .await
    .map_err(|e| anyhow::format_err!("Failed to create websocket: {e}"))?;
    let web3s = Web3::new(websocket);

    let last_gosh_block = get_latest_master_block(&context)
        .await
        .map_err(|e| anyhow::format_err!("Failed latest master block: {e}"))?;

    let last_eth_block = read_block(&web3s, BlockId::Number(BlockNumber::Latest)).await?;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "gosh": last_gosh_block,
            "eth": {
                "hash": last_eth_block.hash,
                "number": last_eth_block.number
            }
        }))
        .map_err(|e| anyhow::format_err!("Failed to serialize last block: {e}"))?
    );
    Ok(())
}
