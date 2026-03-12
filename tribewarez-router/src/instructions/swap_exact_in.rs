use crate::errors::RouterError;
use crate::state::{RouteConfig, SwapRoute};
use anchor_lang::prelude::*;
use anchor_spl::token::Token;

#[derive(Accounts)]
pub struct SwapExactIn<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub route: Account<'info, SwapRoute>,

    #[account(mut)]
    pub config: Account<'info, RouteConfig>,

    pub token_program: Program<'info, Token>,
}

pub fn swap_exact_in(
    ctx: Context<SwapExactIn>,
    amount_in: u64,
    min_amount_out: u64,
    path: Vec<Pubkey>,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let route = &ctx.accounts.route;

    require!(config.is_enabled(), RouterError::InvalidRoute);
    require!(amount_in > 0, RouterError::ZeroAmount);

    let path_len = path.len();
    require!(path_len >= 2, RouterError::InvalidSwapPath);
    require!(
        (path_len - 1) as u8 <= config.max_hops,
        RouterError::RouteTooLong
    );

    require!(path[0] == route.from_token, RouterError::InvalidSwapPath);
    require!(
        path[path_len - 1] == route.to_token,
        RouterError::InvalidSwapPath
    );

    let output_amount = route.calculate_output(amount_in)?;
    require!(
        output_amount >= min_amount_out,
        RouterError::SlippageExceeded
    );

    Ok(())
}
