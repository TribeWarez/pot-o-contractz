//! Update token metadata

use crate::errors::TokenError;
use crate::state::TokenMint;
use anchor_lang::prelude::*;

/// Handle update_metadata instruction
pub fn handle(
    ctx: Context<crate::UpdateMetadata>,
    name: Option<String>,
    symbol: Option<String>,
    uri: Option<String>,
) -> Result<()> {
    let mint = &mut ctx.accounts.mint;

    // Check authority
    require!(
        ctx.accounts.mint_authority.key() == mint.mint_authority,
        TokenError::InvalidMintAuthority
    );

    // Update name if provided
    if let Some(new_name) = name {
        require!(!new_name.is_empty(), TokenError::InvalidTokenAmount);
        mint.name = new_name;
    }

    // Update symbol if provided
    if let Some(new_symbol) = symbol {
        require!(!new_symbol.is_empty(), TokenError::InvalidTokenAmount);
        mint.symbol = new_symbol;
    }

    // Update URI if provided
    if let Some(new_uri) = uri {
        require!(!new_uri.is_empty(), TokenError::InvalidTokenAmount);
        mint.uri = Some(new_uri);
    }

    Ok(())
}
