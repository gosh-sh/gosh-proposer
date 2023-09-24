use common::{eth::transfer::{TransferProposal, Transfer}, gosh::{helper::load_keys, call_function, call_getter}};
use ton_client::ClientContext;
use std::{str::FromStr, sync::Arc, env};
use web3::types::H256;

pub async fn find_proposal(
    context: Arc<ClientContext>,
) -> anyhow::Result<TransferProposal> {
    let abi_path = "../contracts/l2/checker.abi.json";

    let proposal_address = "0:744d576c8fa2946b81b780fc7562e2a098d4720f39b7a3df3250b77efcdcc908".to_owned();
    let from_block = web3::types::BlockId::Hash(
        H256::from_str("0xab5e2d0aa8259ca3901ac094d4378d110c891c1c6fc722879dda140f3be5551b")?
    );
    let till_block = web3::types::BlockId::Hash(
        H256::from_str("0x312ba32443c097756498d345b8232d33d86896c384a5a293c11716c2628102ab")?
    );
    // let transaction_hash = web3::types::TransactionId::Hash(
    //     H256::from_str("0xa6028e247df8db8929db5b006dd68cee2c8797ac10a2a0c4fe396116870b13bf")?
    // );
    let verifying_xfer = Transfer {
        pubkey: "0x0000000000000000000000000000000000000000000000000000000000000064".to_owned(),
        value: 100000000000000,
        hash: "0xa6028e247df8db8929db5b006dd68cee2c8797ac10a2a0c4fe396116870b13bf".to_owned()
    };
    Ok(TransferProposal {address: proposal_address, from_block, till_block, xfer: verifying_xfer})
}

pub async fn approve_proposal(
    context: Arc<ClientContext>,
    proposal_address: String,
) -> anyhow::Result<()> {
    let checker_contract_address = env::var("CHECKER_ADDRESS")?.to_lowercase();
    let checker_abi = "../contracts/l2/checker.abi.json";
    let proposal_abi = "../contracts/l2/proposal_test.abi.json";
    let key_path = "../tests/keys.json";
    let keys = Some(load_keys(key_path)?);

    // TODO check if proposal is existing yet?
    let proposal_addresses =
        call_getter(&context, &checker_contract_address, checker_abi, "", None).await?;

    call_function(
        &context,
        &proposal_address,
        proposal_abi,
        keys,
        "setVote",
        Some(serde_json::json!({"id": 0})),
    ).await?;
    Ok(())
}