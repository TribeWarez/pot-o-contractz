//! Transfer tokens between accounts

use crate::errors::TokenError;
use crate::state::{TokenAccount, TokenMint};
use anchor_lang::prelude::*;

/// Handle transfer instruction
pub fn handle(ctx: Context<crate::Transfer>, amount: u64) -> Result<()> {
    // Validate amount
    require!(amount > 0, TokenError::InvalidTokenAmount);

    let from_account = &mut ctx.accounts.from_account;

    // Check owner
    require!(
        ctx.accounts.from_owner.key() == from_account.owner,
        TokenError::InvalidOwner
    );

    // Check account not frozen
    require!(!from_account.is_frozen, TokenError::AccountFrozen);

    // Check sufficient balance
    require!(
        from_account.balance >= amount,
        TokenError::InsufficientBalance
    );

    // Check mints match
    require!(
        from_account.mint == ctx.accounts.mint.key(),
        TokenError::InvalidTokenAmount
    );

    // Deduct from source
    from_account.balance = from_account
        .balance
        .checked_sub(amount)
        .ok_or(TokenError::InsufficientBalance)?;

    // Add to destination
    let to_account = &mut ctx.accounts.to_account;
    to_account.owner = ctx.accounts.from_owner.key();
    to_account.mint = ctx.accounts.mint.key();

    if to_account.created_at == 0 {
        to_account.created_at = Clock::get()?.unix_timestamp;
    }

    to_account.balance = to_account
        .balance
        .checked_add(amount)
        .ok_or(TokenError::ArithmeticOverflow)?;

    Ok(())
}
