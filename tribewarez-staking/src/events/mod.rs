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
    pub staker: Pubkey,
    pub amount: u64,
    pub lock_until: i64,
    pub entropy_score: u64,  // v0.2.0
    pub coherence: u64,      // v0.2.0
}

/// Emitted when tokens are unstaked.
#[event]
pub struct Unstaked {
    pub staker: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    pub unlock_bonus: u64,   // v0.2.0: early unlock bonus
}

/// Emitted when rewards are claimed.
#[event]
pub struct RewardsClaimed {
    pub staker: Pubkey,
    pub reward_amount: u64,
    pub timestamp: i64,
    pub multiplier: u64,     // v0.2.0: entropy-based multiplier (1e6 scale)
}

/// Emitted when pool configuration changes.
#[event]
pub struct PoolUpdated {
    pub pool: Pubkey,
    pub new_reward_rate: u64,
    pub new_lock_duration: i64,
    pub timestamp: i64,
}

/// Emitted when stakes are entangled (v0.2.0 only).
#[event]
pub struct StakeEntangled {
    pub stake_account: Pubkey,
    pub pool_id: u32,
    pub entropy_contribution: u64,
    pub pool_efficiency: u64,  // Multiplier in 1e6 scale
}
