use common::checker::get_checker_address;
use common::gosh::helper::EverClient;
use common::helper::abi::{CHECKER_ABI, PROPOSAL_ABI};
use common::helper::deserialize_uint;
use common::{
    elock::transfer::TransferPatch,
    gosh::{call_function, call_getter},
};
use serde::Deserialize;
use serde_json::json;
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
    pub transactions: Vec<TransferPatch>,
    #[serde(deserialize_with = "deserialize_uint")]
    pub index: u128,
    #[serde(deserialize_with = "deserialize_uint")]
    pub need: u128,
}

#[derive(Debug)]
pub struct Proposal {
    pub address: String,
    pub details: ProposalDetails,
    pub validator_id: String,
}

#[derive(Deserialize)]
struct GetValidatorIdResult {
    #[serde(rename = "value0")]
    id: Option<String>,
}

pub async fn find_proposals(context: &EverClient, pubkey: String) -> anyhow::Result<Vec<Proposal>> {
    let checker_address = get_checker_address()?;

    let proposal_addresses: AllProposals = call_getter(
        context,
        &checker_address,
        CHECKER_ABI,
        "getAllProposalAddr",
        None,
    )
    .await
    .map_err(|e| anyhow::format_err!("Failed to call getAllProposalAddr: {e}"))?;

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
                tracing::info!(
                    "Failed to query validator id from proposal {proposal_address}: {e}"
                );
                continue;
            }
        };

        match call_getter::<ProposalDetails>(
            context,
            &proposal_address,
            PROPOSAL_ABI,
            "getDetails",
            None,
        )
        .await
        {
            Ok(proposal_details) => {
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

pub async fn get_validator_id(
    context: &EverClient,
    proposal_address: &str,
    pubkey: &str,
) -> anyhow::Result<String> {
    let id: GetValidatorIdResult = call_getter(
        context,
        proposal_address,
        PROPOSAL_ABI,
        "getValidatorId",
        Some(json!({"pubkey": pubkey})),
    )
    .await
    .map_err(|e| anyhow::format_err!("Failed to call getter getValidatorId: {e}"))?;

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
