//! Governance program state structures

use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
}

impl Default for ProposalStatus {
    fn default() -> Self {
        ProposalStatus::Active
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum VoteType {
    For,
    Against,
    Abstain,
}

impl Default for VoteType {
    fn default() -> Self {
        VoteType::For
    }
}

#[account]
#[derive(Default)]
pub struct Proposal {
    pub proposer: Pubkey,
    pub title: String,
    pub description: String,
    pub execution_data: Vec<u8>,
    pub for_votes: u64,
    pub against_votes: u64,
    pub abstain_votes: u64,
    pub status: ProposalStatus,
    pub vote_start: i64,
    pub vote_end: i64,
    pub executed: bool,
    pub execution_timestamp: i64,
    pub bump: u8,
}

impl Proposal {
    pub const INIT_SPACE: usize =
        32 + 4 + 100 + 4 + 500 + 4 + 32 + 8 + 8 + 8 + 1 + 8 + 8 + 1 + 8 + 1;
}

#[account]
#[derive(Default)]
pub struct Vote {
    pub voter: Pubkey,
    pub proposal: Pubkey,
    pub vote_type: VoteType,
    pub weight: u64,
    pub timestamp: i64,
    pub bump: u8,
}

impl Vote {
    pub const INIT_SPACE: usize = 32 + 32 + 1 + 8 + 8 + 1;
}

#[account]
#[derive(Default)]
pub struct Treasury {
    pub balance: u64,
    pub authorized_spender: Pubkey,
}

impl Treasury {
    pub const INIT_SPACE: usize = 8 + 32;
}
