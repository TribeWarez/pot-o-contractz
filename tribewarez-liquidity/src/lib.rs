use anchor_lang::prelude::*;

declare_id!("7S7vHkCcA3x4FxFN1SPvX5LMFkrvGxjghtXn6r8xf8wA");

pub mod errors;
pub mod instructions;
pub mod state;

pub use errors::*;
pub use instructions::*;
pub use state::*;

#[program]
pub mod tribewarez_liquidity {
    use super::*;

    pub fn create_pool(
        ctx: Context<CreatePool>,
        token_a: Pubkey,
        token_b: Pubkey,
        fee_bps: u16,
    ) -> Result<()> {
        instructions::create_pool::create_pool(ctx, token_a, token_b, fee_bps)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
        min_shares: u64,
    ) -> Result<()> {
        instructions::add_liquidity::add_liquidity(ctx, amount_a, amount_b, min_shares)
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        shares: u64,
        min_a: u64,
        min_b: u64,
    ) -> Result<()> {
        instructions::remove_liquidity::remove_liquidity(ctx, shares, min_a, min_b)
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        min_out: u64,
        token_from: Pubkey,
    ) -> Result<()> {
        instructions::swap::swap(ctx, amount_in, min_out, token_from)
    }

    pub fn update_fee(ctx: Context<UpdateFee>, new_fee_bps: u16) -> Result<()> {
        instructions::update_fee::update_fee(ctx, new_fee_bps)
    }
}
