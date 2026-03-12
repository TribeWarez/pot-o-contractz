use crate::errors::GovernanceError;
use crate::state::{Proposal, ProposalStatus, Vote, VoteType};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(
        mut,
        constraint = proposal.status == ProposalStatus::Active @GovernanceError::ProposalNotActive
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        init,
        payer = voter,
        space = 8 + Vote::INIT_SPACE,
        seeds = [b"vote", proposal.key().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub vote: Account<'info, Vote>,

    pub system_program: Program<'info, System>,
}

pub fn cast_vote(ctx: Context<CastVote>, vote_type: VoteType, weight: u64) -> Result<()> {
    require!(weight > 0, GovernanceError::InvalidVoteWeight);

    let proposal = &mut ctx.accounts.proposal;
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp < proposal.vote_end,
        GovernanceError::VotingEnded
    );

    match vote_type {
        VoteType::For => {
            proposal.for_votes = proposal
                .for_votes
                .checked_add(weight)
                .ok_or(error!(GovernanceError::ArithmeticOverflow))?;
        }
        VoteType::Against => {
            proposal.against_votes = proposal
                .against_votes
                .checked_add(weight)
                .ok_or(error!(GovernanceError::ArithmeticOverflow))?;
        }
        VoteType::Abstain => {
            proposal.abstain_votes = proposal
                .abstain_votes
                .checked_add(weight)
                .ok_or(error!(GovernanceError::ArithmeticOverflow))?;
        }
    }

    let vote = &mut ctx.accounts.vote;
    vote.voter = ctx.accounts.voter.key();
    vote.proposal = proposal.key();
    vote.vote_type = vote_type;
    vote.weight = weight;
    vote.timestamp = clock.unix_timestamp;
    vote.bump = ctx.bumps.vote;

    Ok(())
}
