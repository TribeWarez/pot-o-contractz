use crate::errors::GovernanceError;
use crate::state::{Proposal, ProposalStatus};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(
        init,
        payer = proposer,
        space = 8 + Proposal::INIT_SPACE,
        seeds = [b"proposal", proposer.key().as_ref()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    pub system_program: Program<'info, System>,
}

pub fn create_proposal(
    ctx: Context<CreateProposal>,
    title: String,
    description: String,
    execution_data: Vec<u8>,
) -> Result<()> {
    require!(!title.is_empty(), GovernanceError::InvalidTitle);
    require!(title.len() <= 100, GovernanceError::TitleTooLong);
    require!(
        description.len() <= 500,
        GovernanceError::DescriptionTooLong
    );

    let proposal = &mut ctx.accounts.proposal;
    proposal.proposer = ctx.accounts.proposer.key();
    proposal.title = title;
    proposal.description = description;
    proposal.execution_data = execution_data;
    proposal.for_votes = 0;
    proposal.against_votes = 0;
    proposal.abstain_votes = 0;
    proposal.status = ProposalStatus::Active;
    proposal.vote_start = Clock::get()?.unix_timestamp;
    proposal.vote_end = proposal.vote_start + 86400 * 3; // 3 days
    proposal.executed = false;
    proposal.bump = ctx.bumps.proposal;

    Ok(())
}
