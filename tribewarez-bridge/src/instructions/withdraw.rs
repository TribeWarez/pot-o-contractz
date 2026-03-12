use crate::errors::BridgeError;
use crate::state::BridgeVault;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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

pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    require!(!vault.is_paused, BridgeError::BridgePaused);
    require!(amount > 0, BridgeError::InsufficientBalance);
    require!(
        vault.collateral_balance >= amount,
        BridgeError::InsufficientVaultBalance
    );

    let (fee_amount, withdraw_amount) = vault.calculate_fee(amount)?;

    let token_a = vault.token_a;
    let token_b = vault.token_b;
    let bump = vault.bump;

    vault.burn_wrapped(withdraw_amount)?;
    vault.withdraw(withdraw_amount)?;
    vault.collect_fee(fee_amount)?;
    vault.validate_collateral()?;

    let transfer_cpi = Transfer {
        from: ctx.accounts.vault_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.vault.to_account_info(),
    };
    let seeds = &[b"vault", token_a.as_ref(), token_b.as_ref(), &[bump]];
    let signer = &[&seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_cpi,
        signer,
    );
    anchor_spl::token::transfer(cpi_ctx, withdraw_amount)?;

    Ok(())
}
