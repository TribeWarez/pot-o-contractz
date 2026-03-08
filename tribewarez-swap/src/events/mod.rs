use anchor_lang::prelude::*;

/// Emitted when a liquidity pool is initialized.
#[event]
pub struct PoolInitialized {
    pub pool: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub lp_mint: Pubkey,
}

/// Emitted when liquidity is provided.
#[event]
pub struct LiquidityAdded {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub lp_tokens: u64,
}

/// Emitted when liquidity is removed.
#[event]
pub struct LiquidityRemoved {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub lp_tokens: u64,
}

/// Emitted when a swap is executed.
#[event]
pub struct Swapped {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub token_in: Pubkey,
    pub token_out: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee: u64,
}

/// Emitted when a swap quote is generated.
#[event]
pub struct SwapQuote {
    pub pool: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee: u64,
    pub price_impact_bps: u64,
}

/// Emitted when fees are withdrawn.
#[event]
pub struct FeesWithdrawn {
    pub pool: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
}
