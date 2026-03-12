use crate::errors::LiquidityError;
use crate::state::LiquidityPool;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, LiquidityPool>,

    pub system_program: Program<'info, System>,
}

pub fn swap(ctx: Context<Swap>, amount_in: u64, min_out: u64, token_from: Pubkey) -> Result<()> {
    require!(amount_in > 0, LiquidityError::ZeroAmount);

    let pool = &mut ctx.accounts.pool;

    pool.validate_pair(pool.token_a, pool.token_b)?;

    let output_amount = pool.calculate_swap_output(amount_in, token_from)?;
    require!(output_amount >= min_out, LiquidityError::SlippageExceeded);

    // Update reserves
    if token_from == pool.token_a {
        pool.reserve_a = pool
            .reserve_a
            .checked_add(amount_in)
            .ok_or(error!(LiquidityError::ArithmeticOverflow))?;
        pool.reserve_b = pool
            .reserve_b
            .checked_sub(output_amount)
            .ok_or(error!(LiquidityError::InsufficientLiquidity))?;
    } else if token_from == pool.token_b {
        pool.reserve_b = pool
            .reserve_b
            .checked_add(amount_in)
            .ok_or(error!(LiquidityError::ArithmeticOverflow))?;
        pool.reserve_a = pool
            .reserve_a
            .checked_sub(output_amount)
            .ok_or(error!(LiquidityError::InsufficientLiquidity))?;
    }

    Ok(())
}
