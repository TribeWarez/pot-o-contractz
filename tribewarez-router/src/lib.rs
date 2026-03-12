use anchor_lang::prelude::*;

declare_id!("6R6vHkCcA3x4FxFN1SPvX5LMFkrvGxjghtXn6r8xf8wZ");

pub mod errors;
pub mod instructions;
pub mod state;

pub use errors::*;
pub use instructions::*;
pub use state::*;

#[program]
pub mod tribewarez_router {
    use super::*;

    pub fn swap(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64) -> Result<()> {
        instructions::swap::swap(ctx, amount_in, min_amount_out)
    }

    pub fn swap_exact_in(
        ctx: Context<SwapExactIn>,
        amount_in: u64,
        min_amount_out: u64,
        path: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::swap_exact_in::swap_exact_in(ctx, amount_in, min_amount_out, path)
    }

    pub fn get_price(ctx: Context<GetPrice>, amount_in: u64) -> Result<SwapQuote> {
        instructions::get_price::get_price(ctx, amount_in)
    }
}
