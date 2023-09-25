use crate::eth::proposal::{create_proposal, get_proposals, vote_for_withdrawal};

pub async fn check_proposals_and_accept() -> anyhow::Result<()> {
    create_proposal().await?;
    let current_proposals = get_proposals().await?;
    for proposal in current_proposals {
        // TODO: add check of proposal data
        vote_for_withdrawal(proposal.proposal_key).await?;
    }
    Ok(())
}