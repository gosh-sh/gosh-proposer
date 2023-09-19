use crate::gosh::helper::EverClient;
use std::sync::Arc;
use ton_client::abi::{encode_message, Abi, CallSet, ParamsOfEncodeMessage, Signer};
use ton_client::net::{query_collection, ParamsOfQueryCollection};
use ton_client::tvm::{run_tvm, ParamsOfRunTvm};

pub mod helper;

pub async fn call_getter(
    context: &EverClient,
    address: &str,
    abi_path: &str,
    function_name: &str,
    args: Option<serde_json::Value>,
) -> anyhow::Result<serde_json::Value> {
    tracing::trace!("call_getter: address={address}, abi_path={abi_path}, function_name={function_name}, args={args:?}");
    let filter = Some(serde_json::json!({
        "id": { "eq": address }
    }));
    let query = query_collection(
        Arc::clone(context),
        ParamsOfQueryCollection {
            collection: "accounts".to_owned(),
            filter,
            result: "boc".to_owned(),
            limit: Some(1),
            order: None,
        },
    )
    .await
    .map(|r| r.result)?;

    if query.is_empty() {
        anyhow::bail!(
            "account with address {} not found. Was trying to call {}",
            address,
            function_name,
        );
    }
    let account_boc = &query[0]["boc"].as_str();
    if account_boc.is_none() {
        anyhow::bail!("account with address {} does not contain boc", address,);
    }
    let call_set = match args {
        Some(value) => CallSet::some_with_function_and_input(function_name, value),
        None => CallSet::some_with_function(function_name),
    };

    let abi_json = std::fs::read_to_string(abi_path)?;
    let abi = Abi::Json(abi_json);

    let encoded = encode_message(
        Arc::clone(context),
        ParamsOfEncodeMessage {
            abi: abi.clone(),
            address: Some(String::from(address.clone())),
            call_set,
            signer: Signer::None,
            deploy_set: None,
            processing_try_index: None,
            signature_id: None,
        },
    )
    .await
    .map_err(|e| anyhow::format_err!("Failed to encode message: {e}"))?;

    let result = run_tvm(
        Arc::clone(context),
        ParamsOfRunTvm {
            message: encoded.message,
            account: account_boc.unwrap().to_string(),
            abi: Some(abi.clone()),
            boc_cache: None,
            execution_options: None,
            return_updated_account: None,
        },
    )
    .await
    .map(|r| r.decoded.unwrap())
    .map(|r| r.output.unwrap())
    .map_err(|e| anyhow::format_err!("run_local failed: {e}"))?;

    Ok(result)
}
