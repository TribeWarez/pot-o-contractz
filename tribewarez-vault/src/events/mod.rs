use anchor_lang::prelude::*;

/// Emitted when the treasury is initialized.
#[event]
pub struct TreasuryInitialized {
    pub treasury: Pubkey,
    pub authority: Pubkey,
    pub token_mint: Pubkey,
}

/// Emitted when a vault is created.
#[event]
pub struct VaultCreated {
    pub vault: Pubkey,
    pub owner: Pubkey,
    pub name: String,
    pub lock_until: i64,
    pub entropy_score: u64,  // v0.2.0
}

/// Emitted when tokens are deposited.
#[event]
pub struct Deposited {
    pub vault: Pubkey,
    pub amount: u64,
    pub balance_after: u64,
    pub timestamp: i64,
}

/// Emitted when tokens are withdrawn.
#[event]
pub struct Withdrawn {
    pub vault: Pubkey,
    pub amount: u64,
    pub fee: u64,          // v0.2.0: early withdrawal fee
    pub balance_after: u64,
    pub timestamp: i64,
}

/// Emitted when a vault is unlocked (v0.2.0 only).
#[event]
pub struct VaultUnlocked {
    pub vault: Pubkey,
    pub unlock_time: i64,
    pub unlock_probability: u64,  // How probable this unlock was (1e6 scale)
}
