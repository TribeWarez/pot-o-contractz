//! Mint new tokens

use crate::errors::TokenError;
use crate::state::{TokenAccount, TokenMint};
use anchor_lang::prelude::*;

/// Handle mint instruction
pub fn handle(ctx: Context<crate::Mint>, amount: u64) -> Result<()> {
    // Validate amount
    require!(amount > 0, TokenError::InvalidTokenAmount);

    let mint = &mut ctx.accounts.mint;

    // Check if mint authority matches
    require!(
        ctx.accounts.mint_authority.key() == mint.mint_authority,
        TokenError::InvalidMintAuthority
    );

    // Check supply cap
    require!(mint.can_mint(amount), TokenError::SupplyCapExceeded);

    // Check for overflow
    mint.total_supply = mint
        .total_supply
        .checked_add(amount)
        .ok_or(TokenError::SupplyOverflow)?;

    mint.total_minted = mint
        .total_minted
        .checked_add(amount)
        .ok_or(TokenError::SupplyOverflow)?;

    // Update token account
    let token_account = &mut ctx.accounts.token_account;
    token_account.owner = ctx.accounts.authority.key();
    token_account.mint = mint.key();
    token_account.created_at = Clock::get()?.unix_timestamp;

    token_account.balance = token_account
        .balance
        .checked_add(amount)
        .ok_or(TokenError::ArithmeticOverflow)?;

    Ok(())
}
