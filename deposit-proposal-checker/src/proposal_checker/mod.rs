use std::env;
use common::gosh::helper::create_client;
use web3::transports::WebSocket;
use web3::Web3;

use crate::eth::validate::validate_proposal;
use crate::gosh::proposal::{approve_proposal, find_proposals, Proposal, ProposalDetails};

pub async fn check_proposals() -> anyhow::Result<()> {
    let gosh_client = create_client()?;
    // let proposals = find_proposals(&gosh_client).await?;
    let proposals = vec![
        Proposal {
            address: "".to_string(),
            details: ProposalDetails {
                hash: "0x502825da41e0e7252f6afbb1443239f23e74241b1d91672c2cc3bc6a0c410565".to_string(),
                new_hash: "0x312ba32443c097756498d345b8232d33d86896c384a5a293c11716c2628102ab".to_string(),
                transactions: vec![],
                index: 0,
                need: 0,
            }
        }
    ];
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

