//! Governance program errors

use anchor_lang::prelude::*;

#[error_code]
pub enum GovernanceError {
    #[msg("Invalid proposal")]
    InvalidProposal,

    #[msg("Voting not active")]
    VotingNotActive,

    #[msg("Insufficient voting power")]
    InsufficientVotingPower,

    #[msg("Unauthorized operation")]
    Unauthorized,
}
