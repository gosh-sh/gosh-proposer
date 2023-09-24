use common::gosh::helper::create_client;
use std::env;
use web3::transports::WebSocket;
use web3::Web3;

use crate::eth::validate::validate_proposal;
use crate::gosh::proposal::{approve_proposal, find_proposals};

pub async fn check_proposals() -> anyhow::Result<()> {
    let gosh_client = create_client()?;
    let proposals = find_proposals(&gosh_client).await?;
    let websocket = WebSocket::new(&env::var("ETH_NETWORK")?).await?;
    let web3s = Web3::new(websocket);

    for proposal in proposals {
        let address = proposal.address.clone();
        let index = proposal.details.index;
        validate_proposal(&web3s, proposal).await?;
        approve_proposal(&gosh_client, address, index).await?
    }
    Ok(())
}
