use crate::errors::LiquidityError;
use crate::state::LiquidityPool;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, LiquidityPool>,

    pub system_program: Program<'info, System>,
}

pub fn remove_liquidity(
    ctx: Context<RemoveLiquidity>,
    shares: u64,
    min_a: u64,
    min_b: u64,
) -> Result<()> {
    require!(shares > 0, LiquidityError::ZeroAmount);

    let pool = &mut ctx.accounts.pool;

    // Assume total shares from LP mint supply (simplified)
    let total_shares = pool.reserve_a.saturating_add(pool.reserve_b);
    require!(total_shares > 0, LiquidityError::InsufficientLiquidity);

    let (amount_a, amount_b) = pool.calculate_withdraw_amounts(shares, total_shares)?;
    require!(
        amount_a >= min_a && amount_b >= min_b,
        LiquidityError::SlippageExceeded
    );

    pool.update_reserves(amount_a, amount_b, false)?;

    Ok(())
}
