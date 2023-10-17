use common::eth::create_web3_socket;
use common::gosh::helper::{create_client, load_keys};
use std::env;

use proposal::{approve_proposal, find_proposals};
use validate::validate_proposal;

mod proposal;
mod validate;

pub async fn check_proposals() -> anyhow::Result<()> {
    let gosh_client = create_client()?;
    let key_path = env::var("VALIDATORS_KEY_PATH")
        .map_err(|e| anyhow::format_err!("Failed to get end VALIDATORS_KEY_PATH : {e}"))?;

    let keys = load_keys(&key_path)
        .map_err(|e| anyhow::format_err!("Failed to load validator GOSH keys: {e}"))?;
    let pubkey = format!("0x{}", keys.public);
    let keys = Some(keys);

    let proposals = find_proposals(&gosh_client, pubkey).await?;

    let web3s = create_web3_socket().await?;

    for proposal in proposals {
        let address = proposal.address.clone();
        let id = proposal.validator_id.clone();
        match validate_proposal(&web3s, proposal).await {
            Ok(()) => match approve_proposal(&gosh_client, address, &id, keys.clone()).await {
                Ok(()) => {}
                Err(e) => {
                    tracing::info!("Proposal approval failed: {e}");
                }
            },
            Err(e) => {
                tracing::info!("Proposal {} validation failed: {e}", address);
            }
        }
    }
    Ok(())
}
