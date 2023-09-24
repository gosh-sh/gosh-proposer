use std::env;
use common::gosh::helper::create_client;
use web3::transports::WebSocket;
use web3::Web3;

mod l1;
mod l2;
use l1::validate_transaction;
use l2::{approve_proposal, find_proposal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let websocket = WebSocket::new(&env::var("ETH_NETWORK")?).await?;
    let web3s = Web3::new(websocket);

    let gosh_client = create_client()?;

    let proposal = find_proposal(gosh_client.clone()).await?;
    let result = validate_transaction(&web3s, &proposal).await?;

    if result {
        approve_proposal(gosh_client, proposal.address).await?
    }
    Ok(())
}