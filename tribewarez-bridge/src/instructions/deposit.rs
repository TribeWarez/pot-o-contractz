use crate::errors::BridgeError;
use crate::state::BridgeVault;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub vault: Account<'info, BridgeVault>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @BridgeError::UnauthorizedVaultAuthority,
        constraint = user_token_account.mint == vault.token_a @BridgeError::InvalidTokenPair
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_token_account.owner == vault.key() @BridgeError::UnauthorizedVaultAuthority
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    require!(!vault.is_paused, BridgeError::BridgePaused);
    require!(amount > 0, BridgeError::InsufficientBalance);
    require!(
        ctx.accounts.user_token_account.amount >= amount,
        BridgeError::InsufficientBalance
    );

    // Calculate fee
    let (fee_amount, deposit_amount) = vault.calculate_fee(amount)?;

    // Transfer tokens from user to vault
    let transfer_cpi = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.vault_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_cpi);
    anchor_spl::token::transfer(cpi_ctx, amount)?;

    // Update vault state
    vault.deposit(deposit_amount)?;
    vault.collect_fee(fee_amount)?;
    vault.mint_wrapped(deposit_amount)?;

    // Ensure collateral is still backed
    vault.validate_collateral()?;

    Ok(())
}
