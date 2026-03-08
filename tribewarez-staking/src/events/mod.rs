use anchor_lang::prelude::*;

/// Emitted when a staking pool is initialized.
#[event]
pub struct PoolInitialized {
    pub pool: Pubkey,
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub reward_rate: u64,
    pub lock_duration: i64,
}

/// Emitted when tokens are staked.
#[event]
pub struct Staked {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub amount: u64,
    pub total_staked: u64,
    pub unlock_time: i64,
}

/// Emitted when tokens are unstaked.
#[event]
pub struct Unstaked {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub amount: u64,
    pub remaining_stake: u64,
}

/// Emitted when rewards are claimed.
#[event]
pub struct RewardsClaimed {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub amount: u64,
    pub total_claimed: u64,
}

/// Emitted when pool configuration changes.
#[event]
pub struct PoolUpdated {
    pub pool: Pubkey,
    pub reward_rate: u64,
    pub lock_duration: i64,
    pub is_active: bool,
}

/// Emitted when stakes are entangled (v0.2.0 only).
#[event]
pub struct StakeEntangled {
    pub stake_account: Pubkey,
    pub pool_id: u32,
    pub entropy_contribution: u64,
    pub pool_efficiency: u64, // Multiplier in 1e6 scale
}
