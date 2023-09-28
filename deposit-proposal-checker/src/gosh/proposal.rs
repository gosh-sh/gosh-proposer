use common::gosh::helper::EverClient;
use common::helper::abi::{CHECKER_ABI, PROPOSAL_ABI};
use common::helper::deserialize_u128;
use common::{
    eth::transfer::Transfer,
    gosh::{call_function, call_getter, helper::load_keys},
};
use serde::Deserialize;
use serde_json::json;
use std::env;

#[derive(Deserialize)]
struct AllProposals {
    #[serde(rename = "value0")]
    addresses: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProposalDetails {
    pub hash: String,
    #[serde(rename = "newhash")]
    pub new_hash: String,
    pub transactions: Vec<Transfer>,
    #[serde(deserialize_with = "deserialize_u128")]
    pub index: u128,
    #[serde(deserialize_with = "deserialize_u128")]
    pub need: u128,
}

#[derive(Debug)]
pub struct Proposal {
    pub address: String,
    pub details: ProposalDetails,
}

pub async fn find_proposals(context: &EverClient) -> anyhow::Result<Vec<Proposal>> {
    let checker_address = env::var("CHECKER_ADDRESS")
        .map_err(|e| anyhow::format_err!("Failed to get env CHECKER_ADDRESS: {e}"))?;

    let proposal_addresses = call_getter(
        context,
        &checker_address,
        CHECKER_ABI,
        "getAllProposalAddr",
        None,
    )
    .await
    .map_err(|e| anyhow::format_err!("Failed to call getAllProposalAddr: {e}"))?;
    tracing::info!("Get prop addresses res: {:?}", proposal_addresses);
    let proposal_addresses: AllProposals = serde_json::from_value(proposal_addresses)
        .map_err(|e| anyhow::format_err!("Failed to serialize proposal addresses: {e}"))?;

    match proposal_addresses.addresses.len() {
        0 => {
            tracing::info!("There are no proposals in the checker contract");
            std::process::exit(0);
        }
        val => {
            tracing::info!("There are {val} proposals in the checker contract.");
        }
    };
    let mut res = vec![];
    for proposal_address in proposal_addresses.addresses {
        match call_getter(context, &proposal_address, PROPOSAL_ABI, "getDetails", None).await {
            Ok(value) => {
                tracing::info!("Proposal details: {}", value);
                let proposal_details: ProposalDetails = serde_json::from_value(value)
                    .map_err(|e| anyhow::format_err!("Failed to serialize proposal details: {e}"))?;
                res.push(Proposal {
                    address: proposal_address,
                    details: proposal_details,
                });
            },
            Err(e) => {
                tracing::info!("Failed to get details of proposal {}: {e}", proposal_address);
            }
        }
    }
    Ok(res)
}

#[derive(Deserialize)]
struct GetValidatorIdResult {
    #[serde(rename = "value0")]
    id: Option<String>,
}

pub async fn approve_proposal(
    context: &EverClient,
    proposal_address: String,
) -> anyhow::Result<()> {
    let key_path = env::var("VALIDATORS_KEY_PATH")
        .map_err(|e| anyhow::format_err!("Failed to get end VALIDATORS_KEY_PATH : {e}"))?;
    let keys = load_keys(&key_path)
        .map_err(|e| anyhow::format_err!("Failed to load validator GOSH keys: {e}"))?;
    let pubkey = format!("0x{}", keys.public);
    let keys = Some(keys);

    let id_val = call_getter(
        context,
        &proposal_address,
        PROPOSAL_ABI,
        "getValidatorId",
        Some(json!({"pubkey": pubkey})),
    )
    .await
    .map_err(|e| anyhow::format_err!("Failed to call getter getValidatorId: {e}"))?;
    let id: GetValidatorIdResult = serde_json::from_value(id_val)
        .map_err(|e| anyhow::format_err!("Failed to serialize ValidatorId: {e}"))?;
    let id = id
        .id
        .ok_or(anyhow::format_err!("Failed to get id for proposal"))?;

    call_function(
        context,
        &proposal_address,
        PROPOSAL_ABI,
        keys,
        "setVote",
        Some(json!({"id": id})),
    )
    .await
    .map_err(|e| anyhow::format_err!("Failed to call setVote: {e}"))?;
    Ok(())
}
