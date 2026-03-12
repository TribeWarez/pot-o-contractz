//! Burn tokens

use crate::errors::TokenError;
use crate::state::{TokenAccount, TokenMint};
use anchor_lang::prelude::*;

/// Handle burn instruction
pub fn handle(ctx: Context<crate::Burn>, amount: u64) -> Result<()> {
    // Validate amount
    require!(amount > 0, TokenError::InvalidTokenAmount);

    let token_account = &mut ctx.accounts.token_account;

    // Check owner
    require!(
        ctx.accounts.owner.key() == token_account.owner,
        TokenError::InvalidOwner
    );

    // Check account not frozen
    require!(!token_account.is_frozen, TokenError::AccountFrozen);

    // Check sufficient balance
    require!(
        token_account.balance >= amount,
        TokenError::InsufficientBalance
    );

    // Update token account balance
    token_account.balance = token_account
        .balance
        .checked_sub(amount)
        .ok_or(TokenError::InsufficientBalance)?;

    // Update mint
    let mint = &mut ctx.accounts.mint;

    mint.total_supply = mint
        .total_supply
        .checked_sub(amount)
        .ok_or(TokenError::InsufficientBalance)?;

    mint.total_burned = mint
        .total_burned
        .checked_add(amount)
        .ok_or(TokenError::ArithmeticOverflow)?;

    Ok(())
}
