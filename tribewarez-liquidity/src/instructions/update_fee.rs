use crate::errors::LiquidityError;
use crate::state::LiquidityPool;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateFee<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        constraint = pool.admin == admin.key() @LiquidityError::Unauthorized
    )]
    pub pool: Account<'info, LiquidityPool>,
}

pub fn update_fee(ctx: Context<UpdateFee>, new_fee_bps: u16) -> Result<()> {
    require!(new_fee_bps <= 10000, LiquidityError::InvalidFee);

    let pool = &mut ctx.accounts.pool;
    pool.fee_bps = new_fee_bps;

    Ok(())
}
