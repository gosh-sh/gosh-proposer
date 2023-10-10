mod block;
mod call;
pub mod encoder;
pub mod events;
pub mod helper;

pub use block::{read_block, FullBlock};
pub use call::call_function;
use std::env;
use web3::transports::WebSocket;
use web3::Web3;

pub async fn create_web3_socket() -> anyhow::Result<Web3<WebSocket>> {
    let eth_endpoint = env::var("ETH_NETWORK")
        .map_err(|e| anyhow::format_err!("Failed to get ETH_NETWORK env var: {e}"))?;
    tracing::info!("Connecting to the ETH endpoint: {eth_endpoint}");
    let websocket = WebSocket::new(&eth_endpoint)
        .await
        .map_err(|e| anyhow::format_err!("Failed to create websocket: {e}"))?;
    Ok(Web3::new(websocket))
}
