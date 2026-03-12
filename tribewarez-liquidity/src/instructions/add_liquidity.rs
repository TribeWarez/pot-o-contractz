use crate::errors::LiquidityError;
use crate::state::{LiquidityPool, PoolPosition};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + PoolPosition::LEN,
        seeds = [b"position", pool.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub position: Account<'info, PoolPosition>,

    pub system_program: Program<'info, System>,
}

pub fn add_liquidity(
    ctx: Context<AddLiquidity>,
    amount_a: u64,
    amount_b: u64,
    min_shares: u64,
) -> Result<()> {
    require!(amount_a > 0 && amount_b > 0, LiquidityError::ZeroAmount);

    let pool = &mut ctx.accounts.pool;
    let shares = pool.calculate_lp_shares(amount_a, amount_b)?;
    require!(shares >= min_shares, LiquidityError::SlippageExceeded);

    pool.update_reserves(amount_a, amount_b, true)?;

    let position = &mut ctx.accounts.position;
    position.owner = ctx.accounts.user.key();
    position.pool = pool.key();
    position.shares = position
        .shares
        .checked_add(shares)
        .ok_or(error!(LiquidityError::ArithmeticOverflow))?;
    position.bump = ctx.bumps.position;

    Ok(())
}
