use common::checker::{get_block_from_checker, get_checker_address};
use common::elock::transfer::Transfer;
use common::elock::{
    get_elock_address, get_last_gosh_block_id, get_storage, COUNTERS_INDEX, TOTAL_SUPPLY_INDEX,
};
use common::eth::{create_web3_socket, read_block};
use common::gosh::block::{get_latest_master_block, get_master_block_seq_no};
use common::gosh::burn::find_burns;
use common::gosh::call_getter;
use common::gosh::helper::create_client;
use common::gosh::message::get_token_wallet_total_mint;
use common::helper::abi::{CHECKER_ABI, ELOCK_ABI, PROPOSAL_ABI, ROOT_ABI};
use common::helper::deserialize_uint;
use common::token_root::{get_root_address, get_root_owner_address, get_wallet_balance};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::json;
use std::collections::HashMap;
use web3::contract::{Contract, Options};
use web3::types::{Address, BlockId, BlockNumber, U256};

const COLLECTED_COMMISSIONS_INDEX: u8 = 0x13;
const ELOCK_WITHDRAWAL_COMMISSION: u128 = 400_000;
const ELOCK_TRANSFER_COMMISSION: u128 = 21_000;

fn round_serialize<S>(val: &u128, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // let eth = wei_to_eth(U256::from(*val));
    // s.serialize_f64(eth)
    s.serialize_u128(*val)
}

#[derive(Serialize)]
struct Telemetry {
    glock_eth_block: u64,
    last_eth_block: u64,
    eth_block_diff: u64,

    elock_gosh_block_seq_no: u128,
    elock_gosh_block_id: String,
    last_gosh_block_seq_no: u128,
    last_gosh_block_id: String,
    gosh_block_diff: u128,

    queued_burns_cnt: usize,
    #[serde(serialize_with = "round_serialize")]
    queued_burns_total_value: u128,

    elock_deposit_counter: u128,
    elock_withdrawal_counter: u128,
    #[serde(serialize_with = "round_serialize")]
    elock_total_supply: u128,

    elock_proposals_cnt: usize,

    glock_proposals_cnt: usize,
    glock_proposals: HashMap<String, u128>,

    #[serde(serialize_with = "round_serialize")]
    elock_balance: u128,
    validators_balances: HashMap<Address, u128>,

    #[serde(serialize_with = "round_serialize")]
    glock_total_supply: u128,

    #[serde(serialize_with = "round_serialize")]
    elock_collected_commissions: u128,

    #[serde(serialize_with = "round_serialize")]
    glock_total_commissions: u128,
    #[serde(serialize_with = "round_serialize")]
    glock_current_commissions: u128,

    current_eth_gas_price: u128,
    #[serde(serialize_with = "round_serialize")]
    current_approximate_elock_commissions: u128,
    current_approximate_elock_commissions_per_person: u128,
}

#[derive(Deserialize)]
struct AllProposals {
    #[serde(rename = "value0")]
    addresses: Vec<String>,
}

#[derive(Deserialize)]
struct GetTotalSupplyResult {
    #[serde(deserialize_with = "deserialize_uint")]
    #[serde(rename = "value0")]
    value: u128,
}

#[derive(Serialize)]
struct GLockProposal {
    address: String,
    total_value: u128,
}

#[derive(Debug, Deserialize)]
struct ProposalDetails {
    #[serde(rename = "hash")]
    _hash: String,
    #[serde(rename = "newhash")]
    _new_hash: String,
    pub transactions: Vec<Transfer>,
    #[serde(deserialize_with = "deserialize_uint")]
    #[serde(rename = "index")]
    _index: u128,
    #[serde(deserialize_with = "deserialize_uint")]
    #[serde(rename = "need")]
    _need: u128,
}

pub async fn get_telemetry() -> anyhow::Result<()> {
    tracing::info!("Get telemetry");
    let gosh_context = create_client()?;
    let web3s = create_web3_socket().await?;
    // let root_address = get_root_address()?;
    let checker_address = get_checker_address()?;
    let elock_address = get_elock_address()?;
    let elock_abi = web3::ethabi::Contract::load(ELOCK_ABI.as_bytes())?;
    let elock_contract = Contract::new(web3s.eth(), elock_address, elock_abi);

    let block_from_elock = get_last_gosh_block_id(elock_address, &web3s).await?;

    let first_seq_no = get_master_block_seq_no(&gosh_context, &block_from_elock)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get seq no for block from ETH: {e}"))?;

    let current_master_block = get_latest_master_block(&gosh_context)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get latest GOSH block: {e}"))?;

    let gosh_block_diff = current_master_block.seq_no - first_seq_no;

    let burns = find_burns(&gosh_context, first_seq_no, current_master_block.seq_no).await?;

    let queued_burns_cnt = burns.len();
    let mut queued_burns_total_value = 0;
    for burn in burns {
        queued_burns_total_value += burn.value;
    }

    let first_block_hash = get_block_from_checker(&gosh_context, &checker_address).await?;
    let first_block_number = read_block(&web3s, BlockId::Hash(first_block_hash))
        .await?
        .number
        .ok_or(anyhow::format_err!(
            "Failed to read Eth block with hash from GOSH checker: {}",
            web3::helpers::to_string(&first_block_hash)
        ))?;

    let block_id = BlockId::Number(BlockNumber::Finalized);
    let last_block_number = read_block(&web3s, block_id)
        .await?
        .number
        .ok_or(anyhow::format_err!("Failed to read latest Eth block"))?;

    let eth_block_diff = last_block_number - first_block_number;

    let counters = get_storage(&web3s, elock_address, last_block_number, COUNTERS_INDEX).await?;
    tracing::info!("ELock counters: {}", web3::helpers::to_string(&counters));
    let counters_str = web3::helpers::to_string(&counters)
        .replace('"', "")
        .trim_start_matches("0x")
        .to_string();
    let rx_counter = U256::from_str_radix(&counters_str[0..32], 16)?;
    let tx_counter = U256::from_str_radix(&counters_str[32..64], 16)?;

    let total_supply =
        get_storage(&web3s, elock_address, last_block_number, TOTAL_SUPPLY_INDEX).await?;
    let supply_str = web3::helpers::to_string(&total_supply)
        .replace('"', "")
        .trim_start_matches("0x")
        .to_string();
    let total_supply = U256::from_str_radix(&supply_str, 16)?;

    let proposals: Vec<U256> = elock_contract
        .query("getProposalList", (), None, Options::default(), None)
        .await
        .map_err(|e| anyhow::format_err!("Failed to call ELock getter getProposalList: {e}"))?;

    let elock_proposals_cnt = proposals.len();

    let proposal_addresses: AllProposals = call_getter(
        &gosh_context,
        &checker_address,
        CHECKER_ABI,
        "getAllProposalAddr",
        None,
    )
    .await
    .map_err(|e| anyhow::format_err!("Failed to call getAllProposalAddr: {e}"))?;

    let glock_proposals_cnt = proposal_addresses.addresses.len();
    let mut glock_proposals = HashMap::new();
    for proposal_address in proposal_addresses.addresses {
        match call_getter::<ProposalDetails>(
            &gosh_context,
            &proposal_address,
            PROPOSAL_ABI,
            "getDetails",
            None,
        )
        .await
        {
            Ok(proposal_details) => {
                let mut total_value = 0;
                for trans in proposal_details.transactions {
                    total_value += trans.value;
                }
                glock_proposals.insert(proposal_address, total_value);
            }
            Err(e) => {
                tracing::info!(
                    "Failed to get details of proposal {}: {e}",
                    proposal_address
                );
            }
        }
    }

    let elock_balance = web3s
        .eth()
        .balance(elock_address, Some(BlockNumber::Number(last_block_number)))
        .await?;

    let validators: Vec<Address> = elock_contract
        .query("getValidators", (), None, Options::default(), None)
        .await
        .map_err(|e| anyhow::format_err!("Failed to call ELock getter getProposalList: {e}"))?;

    let mut validators_balances = HashMap::new();
    for validator in validators {
        let balance = web3s
            .eth()
            .balance(validator, Some(BlockNumber::Number(last_block_number)))
            .await?;
        validators_balances.insert(validator, balance.as_u128());
    }

    // let glock_total_supply: GetTotalSupplyResult = call_getter(
    //     &gosh_context,
    //     &root_address,
    //     ROOT_ABI,
    //     "getTotalSupply",
    //     None,
    // )
    // .await?;
    // let glock_total_supply = glock_total_supply.value;
    let glock_total_supply = 0;

    let elock_collected_commissions = get_storage(
        &web3s,
        elock_address,
        last_block_number,
        COLLECTED_COMMISSIONS_INDEX,
    )
    .await?;
    let elock_collected_commissions_str = web3::helpers::to_string(&elock_collected_commissions)
        .replace('"', "")
        .trim_start_matches("0x")
        .to_string();
    let elock_collected_commissions =
        U256::from_str_radix(&elock_collected_commissions_str, 16)?.as_u128();

    // let wallet_address = get_root_owner_address(&gosh_context, &root_address).await?;

    let glock_current_commissions = 0; //get_wallet_balance(&gosh_context, &wallet_address).await?;

    let current_eth_gas_price = web3s.eth().gas_price().await?.as_u128();

    let current_approximate_elock_commissions = (ELOCK_WITHDRAWAL_COMMISSION * ((elock_proposals_cnt + 1) as u128) + // + 1  because usually there no proposals and one will definitely be created for withdrawal
        ELOCK_TRANSFER_COMMISSION * (queued_burns_cnt as u128))
        * current_eth_gas_price;

    let current_approximate_elock_commissions_per_person = if queued_burns_cnt != 0 {
        current_approximate_elock_commissions / queued_burns_cnt as u128
    } else {
        0
    };

    let glock_total_commissions = 0;
        // get_token_wallet_total_mint(&gosh_context, &wallet_address).await?;

    let telemetry = Telemetry {
        glock_eth_block: first_block_number.as_u64(),
        last_eth_block: last_block_number.as_u64(),
        eth_block_diff: eth_block_diff.as_u64(),

        elock_gosh_block_seq_no: first_seq_no,
        elock_gosh_block_id: block_from_elock,

        last_gosh_block_seq_no: current_master_block.seq_no,
        last_gosh_block_id: current_master_block.block_id,
        gosh_block_diff,

        queued_burns_cnt,
        queued_burns_total_value,

        elock_deposit_counter: tx_counter.as_u128(),
        elock_withdrawal_counter: rx_counter.as_u128(),
        elock_total_supply: total_supply.as_u128(),

        elock_proposals_cnt,
        glock_proposals_cnt,
        glock_proposals,

        elock_balance: elock_balance.as_u128(),
        validators_balances,

        glock_total_supply,

        elock_collected_commissions,

        glock_total_commissions,
        glock_current_commissions,

        current_eth_gas_price,
        current_approximate_elock_commissions,
        current_approximate_elock_commissions_per_person,
    };

    println!("{}", serde_json::to_string_pretty(&json!(telemetry))?);
    Ok(())
}
