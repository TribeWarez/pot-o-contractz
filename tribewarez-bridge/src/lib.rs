#![allow(unexpected_cfgs, unused_imports, ambiguous_glob_reexports)]

use anchor_lang::prelude::*;

declare_id!("5QvHkCcA3x4FxFN1SPvX5LMFkrvGxjghtXn6r8xf8wY");

pub mod errors;
pub mod instructions;
pub mod state;

pub use errors::*;
pub use instructions::*;
pub use state::*;

#[program]
pub mod tribewarez_bridge {
    use super::*;

    /// Initialize a bridge vault for a token pair
    pub fn initialize_bridge(
        ctx: Context<InitializeBridge>,
        token_a: Pubkey,
        token_b: Pubkey,
        fee_bps: u16,
    ) -> Result<()> {
        instructions::initialize_bridge::initialize_bridge(ctx, token_a, token_b, fee_bps)
    }

    /// Deposit tokens to the bridge vault
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::deposit(ctx, amount)
    }

    /// Withdraw tokens from the bridge vault
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw::withdraw(ctx, amount)
    }

    /// Transfer wrapped tokens between accounts
    pub fn transfer_wrapped(ctx: Context<TransferWrapped>, amount: u64) -> Result<()> {
        instructions::transfer::transfer_wrapped(ctx, amount)
    }

    /// Update bridge vault parameters
    pub fn update_bridge(
        ctx: Context<UpdateBridge>,
        new_fee_bps: Option<u16>,
        new_vault_authority: Option<Pubkey>,
    ) -> Result<()> {
        instructions::update_bridge::update_bridge(ctx, new_fee_bps, new_vault_authority)
    }

    /// Toggle pause status of the bridge vault
    pub fn toggle_pause(ctx: Context<TogglePause>) -> Result<()> {
        instructions::toggle_pause::toggle_pause(ctx)
    }
}
