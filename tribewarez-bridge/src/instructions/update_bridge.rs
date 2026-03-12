use crate::errors::BridgeError;
use crate::state::BridgeVault;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateBridge<'info> {
    #[account(mut)]
    pub vault_authority: Signer<'info>,

    #[account(
        mut,
        constraint = vault.vault_authority == vault_authority.key() @BridgeError::UnauthorizedVaultAuthority
    )]
    pub vault: Account<'info, BridgeVault>,
}

pub fn update_bridge(
    ctx: Context<UpdateBridge>,
    new_fee_bps: Option<u16>,
    new_vault_authority: Option<Pubkey>,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    if let Some(fee) = new_fee_bps {
        require!(fee <= 10000, BridgeError::InvalidFeeBps);
        vault.fee_bps = fee;
    }

    if let Some(new_authority) = new_vault_authority {
        vault.vault_authority = new_authority;
    }

    Ok(())
}
