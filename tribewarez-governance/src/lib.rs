//! Tribewarez Governance Program
//!
//! Implements governance functionality including proposals, voting, and treasury management.

#![allow(unexpected_cfgs, unused_imports, ambiguous_glob_reexports)]

use anchor_lang::prelude::*;

declare_id!("4K5vHkCcA3x4FxFN1SPvX5LMFkrvGxjghtXn6r8xf8xY");

pub mod errors;
pub mod instructions;
pub mod state;

pub use errors::*;
pub use instructions::*;
pub use state::*;

#[program]
pub mod tribewarez_governance {
    use super::*;

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        title: String,
        description: String,
        execution_data: Vec<u8>,
    ) -> Result<()> {
        instructions::create_proposal::create_proposal(ctx, title, description, execution_data)
    }

    pub fn cast_vote(ctx: Context<CastVote>, vote_type: VoteType, weight: u64) -> Result<()> {
        instructions::cast_vote::cast_vote(ctx, vote_type, weight)
    }

    pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
        instructions::execute_proposal::execute_proposal(ctx)
    }
}
