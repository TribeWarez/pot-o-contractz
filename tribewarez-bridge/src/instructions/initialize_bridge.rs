use crate::errors::BridgeError;
use crate::state::BridgeVault;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(token_a: Pubkey, token_b: Pubkey, fee_bps: u16)]
pub struct InitializeBridge<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + BridgeVault::LEN,
        seeds = [b"vault", token_a.as_ref(), token_b.as_ref()],
        bump
    )]
    pub vault: Account<'info, BridgeVault>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_bridge(
    ctx: Context<InitializeBridge>,
    token_a: Pubkey,
    token_b: Pubkey,
    fee_bps: u16,
) -> Result<()> {
    require!(fee_bps <= 10000, BridgeError::InvalidFeeBps);

    let vault = &mut ctx.accounts.vault;
    vault.vault_authority = ctx.accounts.payer.key();
    vault.token_a = token_a;
    vault.token_b = token_b;
    vault.fee_bps = fee_bps;
    vault.is_paused = false;
    vault.collateral_balance = 0;
    vault.wrapped_supply = 0;
    vault.collected_fees = 0;
    vault.bump = ctx.bumps.vault;

    Ok(())
}
