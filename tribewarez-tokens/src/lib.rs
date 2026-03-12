//! Tribewarez Token Program
//!
//! Implements token minting, burning, and transfers for the three-token ecosystem.
//! Supports AUMCOIN, TRIBECOIN, and RAVECOIN with different economic models.

use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

pub use errors::*;
pub use instructions::*;
pub use state::*;

declare_id!("3K5vHkCcA3x4FxFN1SPvX5LMFkrvGxjghtXn6r8xf8wX");

#[program]
pub mod tribewarez_tokens {
    use super::*;

    /// Initialize a new token mint
    pub fn initialize_mint(
        ctx: Context<InitializeMint>,
        decimals: u8,
        supply_cap: Option<u64>,
        inflation_rate: Option<f64>,
        name: String,
        symbol: String,
    ) -> Result<()> {
        instructions::initialize_mint::handle(
            ctx,
            decimals,
            supply_cap,
            inflation_rate,
            name,
            symbol,
        )
    }

    /// Mint new tokens
    pub fn mint(ctx: Context<Mint>, amount: u64) -> Result<()> {
        instructions::mint::handle(ctx, amount)
    }

    /// Burn tokens
    pub fn burn(ctx: Context<Burn>, amount: u64) -> Result<()> {
        instructions::burn::handle(ctx, amount)
    }

    /// Transfer tokens between accounts
    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        instructions::transfer::handle(ctx, amount)
    }

    /// Update token metadata
    pub fn update_metadata(
        ctx: Context<UpdateMetadata>,
        name: Option<String>,
        symbol: Option<String>,
        uri: Option<String>,
    ) -> Result<()> {
        instructions::update_metadata::handle(ctx, name, symbol, uri)
    }

    /// Set the mint authority
    pub fn set_mint_authority(ctx: Context<SetMintAuthority>, new_authority: Pubkey) -> Result<()> {
        instructions::set_mint_authority::handle(ctx, new_authority)
    }

    /// Freeze a token account
    pub fn freeze_account(ctx: Context<FreezeAccount>) -> Result<()> {
        instructions::freeze_account::handle(ctx)
    }

    /// Thaw a frozen token account
    pub fn thaw_account(ctx: Context<ThawAccount>) -> Result<()> {
        instructions::thaw_account::handle(ctx)
    }
}

// Instruction context definitions

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(init, payer = authority, space = TokenMint::SPACE)]
    pub mint: Account<'info, TokenMint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Mint<'info> {
    #[account(mut)]
    pub mint: Account<'info, TokenMint>,

    #[account(
        init_if_needed,
        payer = authority,
        space = TokenAccount::SPACE,
        owner = crate::ID
    )]
    pub token_account: Account<'info, TokenAccount>,

    #[account(address = mint.mint_authority)]
    pub mint_authority: Signer<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Burn<'info> {
    #[account(mut)]
    pub mint: Account<'info, TokenMint>,

    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,

    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub from_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = payer,
        space = TokenAccount::SPACE,
        owner = crate::ID
    )]
    pub to_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, TokenMint>,

    pub from_owner: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
    #[account(mut)]
    pub mint: Account<'info, TokenMint>,

    #[account(address = mint.mint_authority)]
    pub mint_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetMintAuthority<'info> {
    #[account(mut)]
    pub mint: Account<'info, TokenMint>,

    #[account(address = mint.mint_authority)]
    pub current_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct FreezeAccount<'info> {
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, TokenMint>,

    #[account(address = mint.freeze_authority)]
    pub freeze_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ThawAccount<'info> {
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, TokenMint>,

    #[account(address = mint.freeze_authority)]
    pub freeze_authority: Signer<'info>,
}
