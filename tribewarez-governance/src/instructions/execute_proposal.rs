use crate::errors::GovernanceError;
use crate::state::{Proposal, ProposalStatus};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(mut)]
    pub executor: Signer<'info>,

    #[account(
        mut,
        constraint = !proposal.executed @GovernanceError::AlreadyExecuted
    )]
    pub proposal: Account<'info, Proposal>,
}

pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;

    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp >= proposal.vote_end,
        GovernanceError::VotingNotEnded
    );

    let total_votes = proposal
        .for_votes
        .checked_add(proposal.against_votes)
        .and_then(|v| v.checked_add(proposal.abstain_votes))
        .ok_or(error!(GovernanceError::ArithmeticOverflow))?;

    require!(total_votes > 0, GovernanceError::NoVotes);

    let for_percentage = (proposal.for_votes as u128)
        .checked_mul(10000)
        .and_then(|v| v.checked_div(total_votes as u128))
        .ok_or(error!(GovernanceError::ArithmeticOverflow))? as u64;

    require!(for_percentage >= 5000, GovernanceError::ProposalRejected);

    proposal.status = ProposalStatus::Executed;
    proposal.executed = true;
    proposal.execution_timestamp = clock.unix_timestamp;

    Ok(())
}
