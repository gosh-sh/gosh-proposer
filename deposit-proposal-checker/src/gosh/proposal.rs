use common::gosh::helper::EverClient;
use common::helper::deserialize_u128;
use common::{
    eth::transfer::Transfer,
    gosh::{call_function, call_getter, helper::load_keys},
};
use serde::Deserialize;
use serde_json::json;
use std::env;

const CHECKER_ABI_PATH: &str = "contracts/l2/checker.abi.json";
const PROPOSAL_ABI_PATH: &str = "contracts/l2/proposal.abi.json";

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
    let checker_address = env::var("CHECKER_ADDRESS")?;

    let proposal_addresses = call_getter(
        context,
        &checker_address,
        CHECKER_ABI_PATH,
        "getAllProposalAddr",
        None,
    )
    .await?;
    tracing::info!("Get prop addresses res: {:?}", proposal_addresses);
    let proposal_addresses: AllProposals = serde_json::from_value(proposal_addresses)?;

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
        let proposal_details = call_getter(
            context,
            &proposal_address,
            PROPOSAL_ABI_PATH,
            "getDetails",
            None,
        )
        .await?;
        tracing::info!("Proposal details: {}", proposal_details);
        let proposal_details: ProposalDetails = serde_json::from_value(proposal_details)?;
        res.push(Proposal {
            address: proposal_address,
            details: proposal_details,
        });
    }
    Ok(res)
}

#[derive(Deserialize)]
struct GetValidatorIdResult {
    #[serde(rename = "value0")]
    id: Option<u16>,
}

pub async fn approve_proposal(
    context: &EverClient,
    proposal_address: String,
) -> anyhow::Result<()> {
    let proposal_abi = "contracts/l2/proposal_test.abi.json";
    let key_path = env::var("VALIDATORS_KEY_PATH")?;
    let keys = load_keys(&key_path)?;
    let pubkey = format!("0x{}", keys.public);
    let keys = Some(keys);

    let id_val = call_getter(
        context,
        &proposal_address,
        proposal_abi,
        "getValidatorId",
        Some(json!({"pubkey": pubkey})),
    )
    .await?;
    let id: GetValidatorIdResult = serde_json::from_value(id_val)?;
    let id = id
        .id
        .ok_or(anyhow::format_err!("Failed to get id for proposal"))?;

    call_function(
        context,
        &proposal_address,
        proposal_abi,
        keys,
        "setVote",
        Some(json!({"id": id})),
    )
    .await?;
    Ok(())
}
