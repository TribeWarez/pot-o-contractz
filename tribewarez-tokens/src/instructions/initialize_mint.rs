//! Initialize a new token mint

use crate::errors::TokenError;
use crate::state::TokenMint;
use anchor_lang::prelude::*;

/// Handle initialize_mint instruction
pub fn handle(
    ctx: Context<crate::InitializeMint>,
    decimals: u8,
    supply_cap: Option<u64>,
    inflation_rate: Option<f64>,
    name: String,
    symbol: String,
) -> Result<()> {
    // Validate inputs
    require!(decimals <= 18, TokenError::InvalidDecimals);
    require!(!name.is_empty(), TokenError::InvalidTokenAmount);
    require!(!symbol.is_empty(), TokenError::InvalidTokenAmount);

    // Validate inflation rate if provided
    if let Some(rate) = inflation_rate {
        require!(rate >= 0.0 && rate <= 1.0, TokenError::InvalidTokenAmount);
    }

    let mint = &mut ctx.accounts.mint;

    mint.mint_authority = ctx.accounts.authority.key();
    mint.freeze_authority = ctx.accounts.authority.key();
    mint.treasury_address = ctx.accounts.authority.key();
    mint.decimals = decimals;
    mint.supply_cap = supply_cap;
    mint.total_supply = 0;
    mint.total_minted = 0;
    mint.total_burned = 0;
    mint.inflation_rate = inflation_rate;
    mint.name = name;
    mint.symbol = symbol;
    mint.uri = None;
    mint.created_at = Clock::get()?.unix_timestamp;

    // Validate the configuration
    mint.validate()?;

    Ok(())
}
