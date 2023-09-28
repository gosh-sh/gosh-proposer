use common::gosh::helper::EverClient;
use common::helper::abi::{CHECKER_ABI, PROPOSAL_ABI};
use common::helper::deserialize_u128;
use common::{
    eth::transfer::Transfer,
    gosh::{call_function, call_getter},
};
use serde::Deserialize;
use serde_json::json;
use std::env;
use ton_client::crypto::KeyPair;

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
    pub validator_id: String,
}

pub async fn find_proposals(context: &EverClient, pubkey: String) -> anyhow::Result<Vec<Proposal>> {
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
        let id = match get_validator_id(context, &proposal_address, &pubkey).await {
            Ok(id) => id,
            Err(e) => {
                tracing::info!("Failed to query validator id: {e}");
                continue;
            }
        };

        match call_getter(context, &proposal_address, PROPOSAL_ABI, "getDetails", None).await {
            Ok(value) => {
                tracing::info!("Proposal details: {}", value);
                let proposal_details = match serde_json::from_value::<ProposalDetails>(value) {
                    Ok(data) => data,
                    Err(e) => {
                        tracing::info!("Failed to deserialize proposal details: {e}");
                        continue;
                    }
                };
                res.push(Proposal {
                    address: proposal_address,
                    details: proposal_details,
                    validator_id: id,
                });
            }
            Err(e) => {
                tracing::info!(
                    "Failed to get details of proposal {}: {e}",
                    proposal_address
                );
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

pub async fn get_validator_id(
    context: &EverClient,
    proposal_address: &str,
    pubkey: &str,
) -> anyhow::Result<String> {
    let id_val = call_getter(
        context,
        proposal_address,
        PROPOSAL_ABI,
        "getValidatorId",
        Some(json!({"pubkey": pubkey})),
    )
    .await
    .map_err(|e| anyhow::format_err!("Failed to call getter getValidatorId: {e}"))?;
    let id: GetValidatorIdResult = serde_json::from_value(id_val)
        .map_err(|e| anyhow::format_err!("Failed to serialize ValidatorId: {e}"))?;
    id.id
        .ok_or(anyhow::format_err!("Failed to get id for proposal"))
}

pub async fn approve_proposal(
    context: &EverClient,
    proposal_address: String,
    id: &str,
    keys: Option<KeyPair>,
) -> anyhow::Result<()> {
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
