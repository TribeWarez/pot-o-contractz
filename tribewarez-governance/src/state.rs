//! Governance program state structures

use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Pubkey,
    pub title: String,
    pub description: String,
    pub voting_start: i64,
    pub voting_end: i64,
}

#[account]
#[derive(Default)]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub amount: u64,
    pub direction: u8, // 0 = abstain, 1 = yes, 2 = no
}

#[account]
#[derive(Default)]
pub struct Treasury {
    pub balance: u64,
    pub authorized_spender: Pubkey,
}
