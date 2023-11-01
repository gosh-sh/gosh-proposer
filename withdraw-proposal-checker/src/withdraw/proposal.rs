use common::elock::get_last_gosh_block_id;
use common::eth;
use common::gosh::block::{get_latest_master_block, get_master_block_seq_no};
use common::gosh::burn::{find_burns, Burn};
use common::gosh::helper::EverClient;
use ethereum_types::BigEndianHash;
use std::str::FromStr;
use web3::contract::{Contract, Options};
use web3::ethabi::Token;
use web3::signing::SecretKey;
use web3::transports::WebSocket;
use web3::types::{Address, H256, U256};
use web3::Web3;

#[derive(Debug)]
pub struct ProposalData {
    pub proposal_key: U256,
    pub from: String,
    pub till: String,
    pub transfers: Vec<Burn>,
}

pub async fn vote_for_withdrawal(
    prop_key: U256,
    elock_contract: &Contract<WebSocket>,
    key: &SecretKey,
) -> anyhow::Result<()> {
    let prop_str = web3::helpers::to_string(&H256::from_uint(&prop_key));
    tracing::info!("Vote for proposal: {prop_str}");

    eth::call_function(elock_contract, key, "voteForWithdrawal", prop_key).await
}

pub async fn create_proposal(
    context: &EverClient,
    web3s: &Web3<WebSocket>,
    elock_address: Address,
    elock_contract: &Contract<WebSocket>,
    key: &SecretKey,
) -> anyhow::Result<()> {
    // Read last saved block hash from ELock
    let first_block = get_last_gosh_block_id(elock_address, web3s)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get last GOSH block from ELock: {e}"))?;
    // Get seq_no for the block
    let first_seq_no = get_master_block_seq_no(context, &first_block)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get seq no for block from ETH: {e}"))?;

    // Get last block from the network
    let current_master_block = get_latest_master_block(context)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get latest GOSH block: {e}"))?;

    // Find all burns for the specified period between blocks
    let burns = find_burns(context, first_seq_no, current_master_block.seq_no).await?;
    tracing::info!("burns: {burns:?}");

    if burns.is_empty() {
        tracing::info!("There were no burns, do not create proposal");
        return Ok(());
    }

    // Convert arguments for ETH contract call
    let burns =
        convert_burns(burns).map_err(|e| anyhow::format_err!("Failed to convert burns: {e}"))?;

    let first_block = Token::Uint(
        U256::from_str(&first_block)
            .map_err(|e| anyhow::format_err!("Failed to convert first block to U256: {e}"))?,
    );
    let last_block = Token::Uint(
        U256::from_str(&current_master_block.block_id)
            .map_err(|e| anyhow::format_err!("Failed to convert latest block to U256: {e}"))?,
    );

    // Create proposal in ELock
    tracing::info!("Start call of proposeWithdrawal");
    tracing::info!("{first_block} {last_block} {burns:?}");
    eth::call_function(
        elock_contract,
        key,
        "proposeWithdrawal",
        (first_block, last_block, burns),
    )
    .await
}

pub async fn get_proposals(
    elock_contract: &Contract<WebSocket>,
) -> anyhow::Result<Vec<ProposalData>> {
    // Call ELock getter
    let proposals: Vec<U256> = elock_contract
        .query("getProposalList", (), None, Options::default(), None)
        .await
        .map_err(|e| anyhow::format_err!("Failed to call ELock getter getProposalList: {e}"))?;

    tracing::info!("getProposalList: {proposals:?}");

    // Load proposals data
    let mut res = vec![];
    for proposal in proposals {
        let proposals_data: (U256, U256, Vec<Token>) = elock_contract
            .query(
                "getProposal",
                Token::Uint(proposal),
                None,
                Options::default(),
                None,
            )
            .await
            .map_err(|e| anyhow::format_err!("Failed to call ELock getter getProposal: {e}"))?;
        tracing::info!("{proposals_data:?}");

        // Decode getter result
        let transfers = proposals_data
            .2
            .into_iter()
            .map(|val| {
                let vals = val.into_tuple().unwrap();
                let eth_root = vals[0].clone().into_address().unwrap();
                let dest = vals[1].clone().into_address().unwrap();
                let value = vals[2].clone().into_uint().unwrap();
                let hash = H256::from_uint(&vals[3].clone().into_uint().unwrap());
                let tx_id = web3::helpers::to_string(&hash)
                    .replace('"', "")
                    .trim_start_matches("0x")
                    .to_string();
                Burn {
                    dest: format!("{:?}", dest),
                    value: value.as_u128(),
                    tx_id,
                    eth_root: web3::helpers::to_string(&eth_root).replace('"', ""),
                }
            })
            .collect();
        let from = web3::helpers::to_string(&H256::from_uint(&proposals_data.0))
            .replace('"', "")
            .trim_start_matches("0x")
            .to_string();
        let till = web3::helpers::to_string(&H256::from_uint(&proposals_data.1))
            .replace('"', "")
            .trim_start_matches("0x")
            .to_string();
        res.push(ProposalData {
            proposal_key: proposal,
            from,
            till,
            transfers,
        })
    }

    Ok(res)
}

pub async fn check_proposal(context: &EverClient, proposal: &ProposalData) -> anyhow::Result<()> {
    tracing::info!("check proposal: {}", proposal.proposal_key);
    let start_seq_no = get_master_block_seq_no(context, &proposal.from)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get master block from seq no: {e}"))?;

    let end_seq_no = get_master_block_seq_no(context, &proposal.till)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get master block till seq no: {e}"))?;

    if start_seq_no >= end_seq_no {
        anyhow::bail!("Proposal start block seq_no is greater than end's");
    }

    let burns = find_burns(context, start_seq_no, end_seq_no).await?;
    tracing::info!("Found burns: {burns:?}");
    if proposal.transfers != burns {
        anyhow::bail!("list of burns in proposal is not equal to the actual one");
    }
    Ok(())
}

fn convert_burns(burns: Vec<Burn>) -> anyhow::Result<Vec<Token>> {
    let mut res = vec![];
    for burn in burns {
        let mut tuple = vec![];
        let to = Token::Address(Address::from_str(&burn.dest)?);
        let value = Token::Uint(U256::from(burn.value));
        let tx_is = Token::Uint(U256::from_str(&burn.tx_id)?);
        let eth_root = Token::Address(Address::from_str(&burn.eth_root)?);
        tuple.push(eth_root);
        tuple.push(to);
        tuple.push(value);
        tuple.push(tx_is);
        res.push(Token::Tuple(tuple));
    }
    Ok(res)
}
