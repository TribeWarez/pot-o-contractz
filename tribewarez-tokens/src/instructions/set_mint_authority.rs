//! Set the mint authority

use crate::errors::TokenError;
use crate::state::TokenMint;
use anchor_lang::prelude::*;

/// Handle set_mint_authority instruction
pub fn handle(ctx: Context<crate::SetMintAuthority>, new_authority: Pubkey) -> Result<()> {
    let mint = &mut ctx.accounts.mint;

    // Check current authority
    require!(
        ctx.accounts.current_authority.key() == mint.mint_authority,
        TokenError::InvalidMintAuthority
    );

    // Set new authority
    mint.mint_authority = new_authority;

    Ok(())
}
