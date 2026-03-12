//! Freeze a token account

use crate::errors::TokenError;
use crate::state::{TokenAccount, TokenMint};
use anchor_lang::prelude::*;

/// Handle freeze_account instruction
pub fn handle(ctx: Context<crate::FreezeAccount>) -> Result<()> {
    let mint = &ctx.accounts.mint;

    // Check freeze authority
    require!(
        ctx.accounts.freeze_authority.key() == mint.freeze_authority,
        TokenError::InvalidFreezeAuthority
    );

    let token_account = &mut ctx.accounts.token_account;

    // Check account belongs to this mint
    require!(
        token_account.mint == mint.key(),
        TokenError::InvalidTokenAmount
    );

    // Freeze the account
    token_account.is_frozen = true;

    Ok(())
}
