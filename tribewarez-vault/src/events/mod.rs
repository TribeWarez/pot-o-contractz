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
}

/// Emitted when tokens are deposited.
#[event]
pub struct Deposited {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
}

/// Emitted when tokens are withdrawn.
#[event]
pub struct Withdrawn {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
}

/// Emitted when a vault is unlocked (v0.2.0 only).
#[event]
pub struct VaultUnlocked {
    pub vault: Pubkey,
    pub unlock_time: i64,
    pub unlock_probability: u64, // How probable this unlock was (1e6 scale)
}

/// Emitted when an escrow is created.
#[event]
pub struct EscrowCreated {
    pub escrow: Pubkey,
    pub depositor: Pubkey,
    pub beneficiary: Pubkey,
    pub amount: u64,
    pub release_time: i64,
}

/// Emitted when an escrow is released.
#[event]
pub struct EscrowReleased {
    pub escrow: Pubkey,
    pub beneficiary: Pubkey,
    pub amount: u64,
}

/// Emitted when an escrow is cancelled.
#[event]
pub struct EscrowCancelled {
    pub escrow: Pubkey,
    pub depositor: Pubkey,
    pub amount: u64,
}

/// Emitted when a lock is extended.
#[event]
pub struct LockExtended {
    pub vault: Pubkey,
    pub owner: Pubkey,
    pub new_lock_until: i64,
}
