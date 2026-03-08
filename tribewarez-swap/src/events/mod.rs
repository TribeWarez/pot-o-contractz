use anchor_lang::prelude::*;

/// Emitted when a liquidity pool is initialized.
#[event]
pub struct PoolInitialized {
    pub pool: Pubkey,
    pub authority: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub lp_mint: Pubkey,
}

/// Emitted when liquidity is provided.
#[event]
pub struct LiquidityAdded {
    pub pool: Pubkey,
    pub provider: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub lp_tokens_minted: u64,
    pub pool_coherence: u64,  // v0.2.0
}

/// Emitted when liquidity is removed.
#[event]
pub struct LiquidityRemoved {
    pub pool: Pubkey,
    pub provider: Pubkey,
    pub lp_tokens_burned: u64,
    pub amount_a: u64,
    pub amount_b: u64,
}

/// Emitted when a swap is executed.
#[event]
pub struct SwapExecuted {
    pub pool: Pubkey,
    pub trader: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub swap_fee: u64,
    pub price_impact: u64,  // Basis points
    pub coherence_discount: u64,  // v0.2.0
}

/// Emitted when fees are collected.
#[event]
pub struct FeesCollected {
    pub pool: Pubkey,
    pub protocol_fee: u64,
    pub lp_fee: u64,
    pub timestamp: i64,
}
