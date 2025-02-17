use crate::gosh::helper::{default_callback, CallResult, EverClient};
use serde::de;
use std::sync::Arc;

use ton_client::abi::{encode_message, Abi, CallSet, ParamsOfEncodeMessage, Signer};
use ton_client::crypto::KeyPair;
use ton_client::net::{query_collection, ParamsOfQueryCollection};
use ton_client::processing::{ParamsOfProcessMessage, ResultOfProcessMessage};
use ton_client::tvm::{run_tvm, ParamsOfRunTvm};

pub async fn call_getter<T>(
    context: &EverClient,
    address: &str,
    abi_str: &str,
    function_name: &str,
    args: Option<serde_json::Value>,
) -> anyhow::Result<T>
where
    T: de::DeserializeOwned,
{
    tracing::info!("call_getter: address={address}, function_name={function_name}, args={args:?}");
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
    .map(|r| r.result)
    .map_err(|e| anyhow::format_err!("Failed to query account state: {e}"))?;

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

    let abi = Abi::Json(abi_str.to_string());

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

    tracing::info!("Call getter result: {result:?}");

    serde_json::from_value(result)
        .map_err(|e| anyhow::format_err!("Failed to decode getter result: {e:?}"))
}

pub async fn call_function(
    context: &EverClient,
    address: &str,
    abi_str: &str,
    keys: Option<KeyPair>,
    function_name: &str,
    args: Option<serde_json::Value>,
) -> anyhow::Result<()> {
    tracing::info!(
        "call_function: address={address}, function_name={function_name}"
    );
    tracing::trace!("call args={args:?}");

    let call_set = match args {
        Some(value) => CallSet::some_with_function_and_input(function_name, value),
        None => CallSet::some_with_function(function_name),
    };

    let signer = match keys {
        Some(key_pair) => Signer::Keys { keys: key_pair },
        None => Signer::None,
    };

    let abi = Abi::Json(abi_str.to_string());

    let message_encode_params = ParamsOfEncodeMessage {
        abi: abi.clone(),
        address: Some(String::from(address.clone())),
        call_set,
        signer,
        deploy_set: None,
        processing_try_index: None,
        signature_id: None,
    };

    let sdk_result = ton_client::processing::process_message(
        Arc::clone(context),
        ParamsOfProcessMessage {
            send_events: true,
            message_encode_params,
        },
        default_callback,
    )
    .await;
    if let Err(ref e) = sdk_result {
        tracing::error!("process_message error: {:#?}", e);
    }
    let ResultOfProcessMessage {
        transaction, /* decoded, */
        ..
    } = sdk_result?;
    let call_result: CallResult = serde_json::from_value(transaction)?;
    tracing::info!("trx id: {}", call_result.trx_id);
    match call_result.status {
        3 => Ok(()),
        code => anyhow::bail!("Call ended with error code: {code}"),
    }
}
