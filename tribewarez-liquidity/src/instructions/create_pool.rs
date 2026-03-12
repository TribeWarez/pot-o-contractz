use crate::errors::LiquidityError;
use crate::state::LiquidityPool;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(token_a: Pubkey, token_b: Pubkey)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + LiquidityPool::LEN,
        seeds = [b"pool", token_a.as_ref(), token_b.as_ref()],
        bump
    )]
    pub pool: Account<'info, LiquidityPool>,

    pub system_program: Program<'info, System>,
}

pub fn create_pool(
    ctx: Context<CreatePool>,
    token_a: Pubkey,
    token_b: Pubkey,
    fee_bps: u16,
) -> Result<()> {
    require!(fee_bps <= 10000, LiquidityError::InvalidFee);
    require!(token_a != token_b, LiquidityError::InvalidTokenPair);

    let pool = &mut ctx.accounts.pool;
    pool.token_a = token_a;
    pool.token_b = token_b;
    pool.reserve_a = 0;
    pool.reserve_b = 0;
    pool.lp_token_mint = Pubkey::new_unique();
    pool.fee_bps = fee_bps;
    pool.admin = ctx.accounts.admin.key();
    pool.bump = ctx.bumps.pool;

    Ok(())
}
