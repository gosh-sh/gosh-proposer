use std::env;
use std::sync::Arc;
use std::time::Duration;
use ton_client::net::NetworkQueriesProtocol;
use ton_client::{ClientConfig, ClientContext};

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
