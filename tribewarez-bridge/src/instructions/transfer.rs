use crate::errors::BridgeError;
use crate::state::BridgeVault;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct TransferWrapped<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(mut)]
    pub vault: Account<'info, BridgeVault>,

    #[account(
        mut,
        constraint = sender_token_account.owner == sender.key() @BridgeError::UnauthorizedVaultAuthority,
        constraint = sender_token_account.mint == vault.token_b @BridgeError::InvalidTokenPair
    )]
    pub sender_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = recipient_token_account.mint == vault.token_b @BridgeError::InvalidTokenPair
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn transfer_wrapped(ctx: Context<TransferWrapped>, amount: u64) -> Result<()> {
    let vault = &ctx.accounts.vault;

    require!(!vault.is_paused, BridgeError::BridgePaused);
    require!(amount > 0, BridgeError::InsufficientBalance);
    require!(
        ctx.accounts.sender_token_account.amount >= amount,
        BridgeError::InsufficientBalance
    );

    // Transfer wrapped tokens from sender to recipient
    let transfer_cpi = Transfer {
        from: ctx.accounts.sender_token_account.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.sender.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_cpi);
    anchor_spl::token::transfer(cpi_ctx, amount)?;

    Ok(())
}
