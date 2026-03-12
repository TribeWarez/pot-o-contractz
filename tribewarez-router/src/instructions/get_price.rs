use crate::errors::RouterError;
use crate::state::{RouteConfig, SwapQuote, SwapRoute};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct GetPrice<'info> {
    pub route: Account<'info, SwapRoute>,
    pub config: Account<'info, RouteConfig>,
}

pub fn get_price(ctx: Context<GetPrice>, amount_in: u64) -> Result<SwapQuote> {
    let config = &ctx.accounts.config;
    let route = &ctx.accounts.route;

    require!(config.is_enabled(), RouterError::InvalidRoute);
    require!(amount_in > 0, RouterError::ZeroAmount);

    let output_amount = route.calculate_output(amount_in)?;

    let fee = amount_in
        .checked_mul(route.fee_bps as u64)
        .and_then(|v| v.checked_div(10000))
        .unwrap_or(0);

    let quote = SwapQuote {
        amount_in,
        amount_out: output_amount,
        fee,
        path: vec![route.from_token, route.to_token],
        price_impact: 0,
        timestamp: Clock::get()?.unix_timestamp,
    };

    Ok(quote)
}
