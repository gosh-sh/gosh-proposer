use std::env;
use std::sync::Arc;
use std::time::Duration;
use serde::Deserialize;
use ton_client::net::NetworkQueriesProtocol;
use ton_client::{ClientConfig, ClientContext};
use ton_client::crypto::KeyPair;
use ton_client::processing::ProcessingEvent;

pub type EverClient = Arc<ClientContext>;
static DEFAULT_BLOCKCHAIN_TIMEOUT: Duration = Duration::from_secs(15 * 60);
static MESSAGE_PROCESSING_TIMEOUT: &'static str = "GOSH_MESSAGE_PROCESSING_TIMEOUT_SEC";
static WAIT_FOR_TIMEOUT: &'static str = "GOSH_WAIT_FOR_TIMEOUT_SEC";
static QUERY_TIMEOUT: &'static str = "GOSH_QUERY_TIMEOUT_SEC";

pub fn create_client() -> anyhow::Result<EverClient> {
    let endpoints = env::var("GOSH_ENDPOINTS")?
        .split(',')
        .map(|e| e.to_string())
        .collect::<Vec<String>>();
    tracing::info!("create gosh client. endpoints: {endpoints:?}");

    let message_processing_timeout = env::var(MESSAGE_PROCESSING_TIMEOUT)
        .map(|secs| Duration::from_secs(u64::from_str_radix(&secs, 10).unwrap_or(0)))
        .unwrap_or(DEFAULT_BLOCKCHAIN_TIMEOUT);
    let wait_for_timeout = env::var(WAIT_FOR_TIMEOUT)
        .map(|secs| Duration::from_secs(u64::from_str_radix(&secs, 10).unwrap_or(0)))
        .unwrap_or(DEFAULT_BLOCKCHAIN_TIMEOUT);
    let query_timeout = env::var(QUERY_TIMEOUT)
        .map(|secs| Duration::from_secs(u64::from_str_radix(&secs, 10).unwrap_or(0)))
        .unwrap_or(DEFAULT_BLOCKCHAIN_TIMEOUT);

    let config = ClientConfig {
        network: ton_client::net::NetworkConfig {
            sending_endpoint_count: endpoints.len() as u8,
            endpoints: if endpoints.is_empty() {
                None
            } else {
                Some(endpoints)
            },
            queries_protocol: NetworkQueriesProtocol::HTTP,
            network_retries_count: 5,
            message_retries_count: 10,
            message_processing_timeout: message_processing_timeout.as_millis().try_into()?,
            wait_for_timeout: wait_for_timeout.as_millis().try_into()?,
            query_timeout: query_timeout.as_millis().try_into()?,
            ..Default::default()
        },
        ..Default::default()
    };
    let es_client = ClientContext::new(config)
        .map_err(|e| anyhow::anyhow!("failed to create EverSDK client: {}", e))?;

    Ok(Arc::new(es_client))
}

#[derive(Deserialize, Debug)]
pub struct CallResult {
    #[serde(rename = "id")]
    pub trx_id: String,
    pub status: u8,
    #[serde(with = "ton_sdk::json_helper::uint")]
    total_fees: u64,
    in_msg: String,
    out_msgs: Vec<String>,
}

fn processing_event_to_string(pe: ProcessingEvent) -> String {
    match pe {
        ProcessingEvent::WillSend {
            shard_block_id,
            message_id,
            message: _,
            ..
        } => format!(
            "\nWillSend: {{\n\t\
shard_block_id: \"{shard_block_id}\",\n\t\
message_id: \"{message_id}\"\n}}"
        ),
        ProcessingEvent::DidSend {
            shard_block_id,
            message_id,
            message: _,
            ..
        } => format!(
            "\nDidSend: {{\n\t\
shard_block_id: \"{shard_block_id}\",\n\t\
message_id: \"{message_id}\"\n}}"
        ),
        ProcessingEvent::SendFailed {
            shard_block_id,
            message_id,
            message: _,
            error,
            ..
        } => format!(
            "\nSendFailed: {{\n\t\
shard_block_id: \"{shard_block_id}\",\n\t\
message_id: \"{message_id}\"\n\t\
error: \"{error}\"\n}}"
        ),
        ProcessingEvent::WillFetchNextBlock {
            shard_block_id,
            message_id,
            message: _,
            ..
        } => format!(
            "\nWillFetchNextBlock: {{\n\t\
shard_block_id: \"{shard_block_id}\",\n\t\
message_id: \"{message_id}\"\n}}"
        ),
        ProcessingEvent::FetchNextBlockFailed {
            shard_block_id,
            message_id,
            message: _,
            error,
            ..
        } => format!(
            "\nFetchNextBlockFailed: {{\n\tshard_block_id: \"{shard_block_id}\",\n\t\
message_id: \"{message_id}\"\n\terror: \"{error}\"\n}}"
        ),
        ProcessingEvent::MessageExpired {
            message_id,
            message: _,
            error,
            ..
        } => format!(
            "\nMessageExpired: {{\n\terror: \"{error}\",\n\tmessage_id: \"{message_id}\"\n}}"
        ),
        _ => format!("{:#?}", pe),
    }
}

pub async fn default_callback(pe: ProcessingEvent) {
    tracing::trace!("callback: {}", processing_event_to_string(pe));
}

pub fn load_keys(path: &str) -> anyhow::Result<KeyPair> {
    let data_str = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data_str)
        .map_err(|e| anyhow::format_err!("Failed to load key pair from {path}: {e}"))?)
}