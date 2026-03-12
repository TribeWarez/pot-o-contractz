use crate::errors::BridgeError;
use crate::state::BridgeVault;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TogglePause<'info> {
    #[account(mut)]
    pub vault_authority: Signer<'info>,

    #[account(
        mut,
        constraint = vault.vault_authority == vault_authority.key() @BridgeError::UnauthorizedVaultAuthority
    )]
    pub vault: Account<'info, BridgeVault>,
}

pub fn toggle_pause(ctx: Context<TogglePause>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    if vault.is_paused {
        vault.resume();
    } else {
        vault.pause();
    }

    Ok(())
}
