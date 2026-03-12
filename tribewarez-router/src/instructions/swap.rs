use crate::errors::RouterError;
use crate::state::{RouteConfig, SwapRoute};
use anchor_lang::prelude::*;
use anchor_spl::token::Token;

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub route: Account<'info, SwapRoute>,

    #[account(mut)]
    pub config: Account<'info, RouteConfig>,

    pub token_program: Program<'info, Token>,
}

pub fn swap(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64) -> Result<()> {
    let config = &ctx.accounts.config;
    let route = &ctx.accounts.route;

    require!(config.is_enabled(), RouterError::InvalidRoute);
    require!(amount_in > 0, RouterError::ZeroAmount);

    let output_amount = route.calculate_output(amount_in)?;
    require!(
        output_amount >= min_amount_out,
        RouterError::SlippageExceeded
    );

    Ok(())
}
