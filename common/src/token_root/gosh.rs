use crate::gosh::{call_getter, call_function};
use crate::gosh::helper::EverClient;
use crate::helper::{
    abi::{ROOT_ABI, TOKEN_WALLET_ABI},
    deserialize_uint,
};
use serde::Deserialize;
use serde_json::json;
use crate::helper::abi::CHECKER_ABI;
use crate::token_root::RootData;

#[derive(Deserialize)]
struct GetRootAddrResult {
    #[serde(rename = "value0")]
    address: String,
}

#[derive(Deserialize)]
struct GetNameResult {
    #[serde(rename = "value0")]
    name: String,
}

#[derive(Deserialize)]
struct GetRootPubkeyResult {
    #[serde(rename = "value0")]
    pubkey: String,
}

#[derive(Deserialize)]
struct GetWalletAddressResult {
    #[serde(rename = "value0")]
    address: String,
}

#[derive(Deserialize)]
struct EverAddress {
    #[serde(deserialize_with = "deserialize_uint")]
    #[serde(rename = "workchain_id")]
    _workchain_id: i8,
    #[serde(rename = "address")]
    _address: String,
}

#[derive(Deserialize)]
struct LendOwnerKey {
    #[serde(rename = "dest")]
    _dest: EverAddress,
}

#[derive(Deserialize)]
struct LendOwner {
    #[serde(deserialize_with = "deserialize_uint")]
    #[serde(rename = "lend_balance")]
    _lend_balance: u128,
    #[serde(deserialize_with = "deserialize_uint")]
    #[serde(rename = "lend_finish_time")]
    _lend_finish_time: u32,
    #[serde(rename = "lend_key")]
    _lend_key: LendOwnerKey,
}

#[derive(Deserialize)]
struct Binding {
    #[serde(rename = "flex")]
    _flex: String,
    #[serde(rename = "unsalted_price_code_hash")]
    _unsalted_price_code_hash: String,
}

#[derive(Deserialize)]
struct WalletDetails {
    #[serde(rename = "name")]
    _name: String,
    #[serde(rename = "symbol")]
    _symbol: String,
    #[serde(deserialize_with = "deserialize_uint")]
    #[serde(rename = "decimals")]
    _decimals: u8,
    #[serde(deserialize_with = "deserialize_uint")]
    balance: u128,
    #[serde(deserialize_with = "deserialize_uint")]
    #[serde(rename = "locked")]
    _locked: u128,
    #[serde(rename = "root_pubkey")]
    _root_pubkey: String,
    #[serde(rename = "root_address")]
    _root_address: String,
    #[serde(rename = "wallet_pubkey")]
    _wallet_pubkey: String,
    #[serde(rename = "owner_address")]
    _owner_address: Option<String>,
    #[serde(rename = "lend_pubkey")]
    _lend_pubkey: Option<String>,
    #[serde(rename = "lend_owners")]
    _lend_owners: Vec<LendOwner>,
    #[serde(deserialize_with = "deserialize_uint")]
    #[serde(rename = "lend_balance")]
    _lend_balance: u128,
    #[serde(rename = "binding")]
    _binding: Option<Vec<Binding>>,
    #[serde(rename = "code_hash")]
    _code_hash: String,
    #[serde(deserialize_with = "deserialize_uint")]
    #[serde(rename = "code_depth")]
    _code_depth: u16,
    #[serde(deserialize_with = "deserialize_uint")]
    #[serde(rename = "workchain_id")]
    _workchain_id: i8,
}

pub async fn get_root_owner_address(
    gosh_context: &EverClient,
    root_address: &str,
) -> anyhow::Result<String> {
    let root_owner_key: GetRootPubkeyResult =
        call_getter(gosh_context, root_address, ROOT_ABI, "getRootKey", None).await?;

    let owner_wallet: GetWalletAddressResult = call_getter(
        gosh_context,
        root_address,
        ROOT_ABI,
        "getWalletAddress",
        Some(json!({
            "pubkey": root_owner_key.pubkey,
            "owner": serde_json::Value::Null
        })),
    )
    .await?;

    Ok(owner_wallet.address)
}

pub async fn get_wallet_balance(
    gosh_context: &EverClient,
    wallet_address: &str,
) -> anyhow::Result<u128> {
    let details: WalletDetails = call_getter(
        gosh_context,
        wallet_address,
        TOKEN_WALLET_ABI,
        "getDetails",
        None,
    )
    .await?;
    Ok(details.balance)
}

pub async fn get_root_owner_balance(
    gosh_context: &EverClient,
    root_address: &str,
) -> anyhow::Result<u128> {
    let wallet_address = get_root_owner_address(gosh_context, root_address).await?;
    get_wallet_balance(gosh_context, &wallet_address).await
}

pub async fn get_root_address(
    gosh_context: &EverClient,
    checker_address: &str,
    root_data: &RootData,
) -> anyhow::Result<String> {
    tracing::info!("Get root address: checker_address={checker_address} root_data={root_data:?}");
    let root: GetRootAddrResult = call_getter(
        gosh_context,
        checker_address,
        CHECKER_ABI,
        "getRootAddr",
        Some(json!({"data": root_data})),
    )
        .await?;
    Ok(root.address)
}

pub async fn is_root_active(
    gosh_context: &EverClient,
    checker_address: &str,
    root_data: &RootData,
) -> anyhow::Result<bool> {
    tracing::info!("Is root active: checker_address={checker_address} root_data={root_data:?}");
    let root_address = get_root_address(
        gosh_context,
        checker_address,
        root_data
    ).await?;
    let res: anyhow::Result<GetNameResult> = call_getter(
        gosh_context,
        &root_address,
        ROOT_ABI,
        "getName",
        None,
    ).await;
    match res {
        Err(e) => {
            tracing::info!("Failed to call root getter: {e}");
            Ok(false)
        }
        Ok(res) => {
            assert_eq!(res.name, root_data.name,
                       "Root contract name getter does not match expected");
            Ok(true)
        }
    }
}

pub async fn deploy_root(
    gosh_context: &EverClient,
    checker_address: &str,
    root_data: &RootData,
) -> anyhow::Result<()> {
    let eth_root = web3::helpers::to_string(&root_data.eth_root)
        .replace('"', "")
        .trim_start_matches("0x").to_string();
    let eth_root = format!("0x000000000000000000000000{}", eth_root);
    call_function(
        gosh_context,
        checker_address,
        CHECKER_ABI,
        None,
        "deployRootContract",
        Some(json!({
            "name": root_data.name,
            "symbol": root_data.symbol,
            "decimals": root_data.decimals,
            "ethroot": eth_root
        })),
    ).await
}
