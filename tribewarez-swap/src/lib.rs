use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};

declare_id!("GPGGnKwnvKseSxzPukrNvch1CwYhifTqgj2RdW1P26H3");

// Tribewarez Swap Program
// Constant Product AMM (x * y = k) for PTtC token swaps.
// Supports liquidity provision, swaps, and fee collection.

// Fee configuration (basis points - 10000 = 100%)
const SWAP_FEE_BPS: u64 = 30; // 0.30% swap fee
const PROTOCOL_FEE_BPS: u64 = 5; // 0.05% protocol fee
#[allow(dead_code)]
const LP_FEE_BPS: u64 = 25; // 0.25% to LPs

#[program]
pub mod tribewarez_swap {
    use super::*;

    /// Initialize a new liquidity pool
    pub fn initialize_pool(ctx: Context<InitializePool>, pool_bump: u8) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        pool.authority = ctx.accounts.authority.key();
        pool.token_a_mint = ctx.accounts.token_a_mint.key();
        pool.token_b_mint = ctx.accounts.token_b_mint.key();
        pool.token_a_vault = ctx.accounts.token_a_vault.key();
        pool.token_b_vault = ctx.accounts.token_b_vault.key();
        pool.lp_mint = ctx.accounts.lp_mint.key();
        pool.reserve_a = 0;
        pool.reserve_b = 0;
        pool.total_lp_supply = 0;
        pool.swap_fee_bps = SWAP_FEE_BPS;
        pool.protocol_fee_bps = PROTOCOL_FEE_BPS;
        pool.collected_fees_a = 0;
        pool.collected_fees_b = 0;
        pool.bump = pool_bump;
        pool.is_active = true;
        pool.created_at = Clock::get()?.unix_timestamp;

        emit!(PoolInitialized {
            pool: pool.key(),
            token_a_mint: pool.token_a_mint,
            token_b_mint: pool.token_b_mint,
            lp_mint: pool.lp_mint,
        });

        Ok(())
    }

    /// Add liquidity to the pool
    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
        min_lp_tokens: u64,
    ) -> Result<()> {
        require!(amount_a > 0 && amount_b > 0, SwapError::InvalidAmount);
        require!(ctx.accounts.pool.is_active, SwapError::PoolInactive);

        let pool = &mut ctx.accounts.pool;
        let lp_tokens_to_mint: u64;

        if pool.total_lp_supply == 0 {
            // First liquidity provider - mint sqrt(a * b) LP tokens
            lp_tokens_to_mint = (amount_a as u128)
                .checked_mul(amount_b as u128)
                .ok_or(SwapError::MathOverflow)?
                .integer_sqrt() as u64;

            require!(lp_tokens_to_mint > 0, SwapError::InsufficientLiquidity);
        } else {
            // Calculate proportional LP tokens
            let lp_from_a = (amount_a as u128)
                .checked_mul(pool.total_lp_supply as u128)
                .ok_or(SwapError::MathOverflow)?
                .checked_div(pool.reserve_a as u128)
                .ok_or(SwapError::MathOverflow)? as u64;

            let lp_from_b = (amount_b as u128)
                .checked_mul(pool.total_lp_supply as u128)
                .ok_or(SwapError::MathOverflow)?
                .checked_div(pool.reserve_b as u128)
                .ok_or(SwapError::MathOverflow)? as u64;

            // Use minimum to prevent manipulation
            lp_tokens_to_mint = lp_from_a.min(lp_from_b);
        }

        require!(
            lp_tokens_to_mint >= min_lp_tokens,
            SwapError::SlippageExceeded
        );

        // Transfer token A to pool
        let cpi_accounts_a = Transfer {
            from: ctx.accounts.user_token_a.to_account_info(),
            to: ctx.accounts.token_a_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts_a),
            amount_a,
        )?;

        // Transfer token B to pool
        let cpi_accounts_b = Transfer {
            from: ctx.accounts.user_token_b.to_account_info(),
            to: ctx.accounts.token_b_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts_b),
            amount_b,
        )?;

        // Mint LP tokens to user using PDA signer
        let token_a_mint = pool.token_a_mint;
        let token_b_mint = pool.token_b_mint;
        let seeds = &[
            b"pool",
            token_a_mint.as_ref(),
            token_b_mint.as_ref(),
            &[pool.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts_mint = MintTo {
            mint: ctx.accounts.lp_mint.to_account_info(),
            to: ctx.accounts.user_lp_account.to_account_info(),
            authority: pool.to_account_info(),
        };
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts_mint,
                signer,
            ),
            lp_tokens_to_mint,
        )?;

        // Update pool state
        pool.reserve_a = pool
            .reserve_a
            .checked_add(amount_a)
            .ok_or(SwapError::MathOverflow)?;
        pool.reserve_b = pool
            .reserve_b
            .checked_add(amount_b)
            .ok_or(SwapError::MathOverflow)?;
        pool.total_lp_supply = pool
            .total_lp_supply
            .checked_add(lp_tokens_to_mint)
            .ok_or(SwapError::MathOverflow)?;

        emit!(LiquidityAdded {
            pool: pool.key(),
            user: ctx.accounts.user.key(),
            amount_a,
            amount_b,
            lp_tokens: lp_tokens_to_mint,
        });

        Ok(())
    }

    /// Remove liquidity from the pool
    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        lp_amount: u64,
        min_amount_a: u64,
        min_amount_b: u64,
    ) -> Result<()> {
        require!(lp_amount > 0, SwapError::InvalidAmount);

        let pool = &mut ctx.accounts.pool;

        // Calculate token amounts to return
        let amount_a = (lp_amount as u128)
            .checked_mul(pool.reserve_a as u128)
            .ok_or(SwapError::MathOverflow)?
            .checked_div(pool.total_lp_supply as u128)
            .ok_or(SwapError::MathOverflow)? as u64;

        let amount_b = (lp_amount as u128)
            .checked_mul(pool.reserve_b as u128)
            .ok_or(SwapError::MathOverflow)?
            .checked_div(pool.total_lp_supply as u128)
            .ok_or(SwapError::MathOverflow)? as u64;

        require!(
            amount_a >= min_amount_a && amount_b >= min_amount_b,
            SwapError::SlippageExceeded
        );

        // Burn LP tokens
        let cpi_accounts_burn = Burn {
            mint: ctx.accounts.lp_mint.to_account_info(),
            from: ctx.accounts.user_lp_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts_burn,
            ),
            lp_amount,
        )?;

        // Transfer tokens back to user using PDA signer
        let token_a_mint = pool.token_a_mint;
        let token_b_mint = pool.token_b_mint;
        let seeds = &[
            b"pool",
            token_a_mint.as_ref(),
            token_b_mint.as_ref(),
            &[pool.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts_a = Transfer {
            from: ctx.accounts.token_a_vault.to_account_info(),
            to: ctx.accounts.user_token_a.to_account_info(),
            authority: pool.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts_a,
                signer,
            ),
            amount_a,
        )?;

        let cpi_accounts_b = Transfer {
            from: ctx.accounts.token_b_vault.to_account_info(),
            to: ctx.accounts.user_token_b.to_account_info(),
            authority: pool.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts_b,
                signer,
            ),
            amount_b,
        )?;

        // Update pool state
        pool.reserve_a = pool
            .reserve_a
            .checked_sub(amount_a)
            .ok_or(SwapError::MathOverflow)?;
        pool.reserve_b = pool
            .reserve_b
            .checked_sub(amount_b)
            .ok_or(SwapError::MathOverflow)?;
        pool.total_lp_supply = pool
            .total_lp_supply
            .checked_sub(lp_amount)
            .ok_or(SwapError::MathOverflow)?;

        emit!(LiquidityRemoved {
            pool: pool.key(),
            user: ctx.accounts.user.key(),
            amount_a,
            amount_b,
            lp_tokens: lp_amount,
        });

        Ok(())
    }

    /// Swap token A for token B
    pub fn swap_a_for_b(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64) -> Result<()> {
        require!(amount_in > 0, SwapError::InvalidAmount);
        require!(ctx.accounts.pool.is_active, SwapError::PoolInactive);

        let pool = &mut ctx.accounts.pool;

        // Calculate output using constant product formula with fee
        let amount_out =
            calculate_swap_output(amount_in, pool.reserve_a, pool.reserve_b, pool.swap_fee_bps)?;

        require!(amount_out >= min_amount_out, SwapError::SlippageExceeded);
        require!(
            amount_out < pool.reserve_b,
            SwapError::InsufficientLiquidity
        );

        // Calculate and track fees
        let fee = calculate_fee(amount_in, pool.swap_fee_bps)?;
        let protocol_fee = calculate_fee(amount_in, pool.protocol_fee_bps)?;
        pool.collected_fees_a = pool
            .collected_fees_a
            .checked_add(protocol_fee)
            .ok_or(SwapError::MathOverflow)?;

        // Transfer token A from user to pool
        let cpi_accounts_in = Transfer {
            from: ctx.accounts.user_token_a.to_account_info(),
            to: ctx.accounts.token_a_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts_in,
            ),
            amount_in,
        )?;

        // Transfer token B from pool to user using PDA signer
        let token_a_mint = pool.token_a_mint;
        let token_b_mint = pool.token_b_mint;
        let seeds = &[
            b"pool",
            token_a_mint.as_ref(),
            token_b_mint.as_ref(),
            &[pool.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts_out = Transfer {
            from: ctx.accounts.token_b_vault.to_account_info(),
            to: ctx.accounts.user_token_b.to_account_info(),
            authority: pool.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts_out,
                signer,
            ),
            amount_out,
        )?;

        // Update reserves
        pool.reserve_a = pool
            .reserve_a
            .checked_add(amount_in)
            .ok_or(SwapError::MathOverflow)?;
        pool.reserve_b = pool
            .reserve_b
            .checked_sub(amount_out)
            .ok_or(SwapError::MathOverflow)?;

        emit!(Swapped {
            pool: pool.key(),
            user: ctx.accounts.user.key(),
            token_in: pool.token_a_mint,
            token_out: pool.token_b_mint,
            amount_in,
            amount_out,
            fee,
        });

        Ok(())
    }

    /// Swap token B for token A
    pub fn swap_b_for_a(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64) -> Result<()> {
        require!(amount_in > 0, SwapError::InvalidAmount);
        require!(ctx.accounts.pool.is_active, SwapError::PoolInactive);

        let pool = &mut ctx.accounts.pool;

        // Calculate output using constant product formula with fee
        let amount_out =
            calculate_swap_output(amount_in, pool.reserve_b, pool.reserve_a, pool.swap_fee_bps)?;

        require!(amount_out >= min_amount_out, SwapError::SlippageExceeded);
        require!(
            amount_out < pool.reserve_a,
            SwapError::InsufficientLiquidity
        );

        // Calculate and track fees
        let fee = calculate_fee(amount_in, pool.swap_fee_bps)?;
        let protocol_fee = calculate_fee(amount_in, pool.protocol_fee_bps)?;
        pool.collected_fees_b = pool
            .collected_fees_b
            .checked_add(protocol_fee)
            .ok_or(SwapError::MathOverflow)?;

        // Transfer token B from user to pool
        let cpi_accounts_in = Transfer {
            from: ctx.accounts.user_token_b.to_account_info(),
            to: ctx.accounts.token_b_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts_in,
            ),
            amount_in,
        )?;

        // Transfer token A from pool to user using PDA signer
        let token_a_mint = pool.token_a_mint;
        let token_b_mint = pool.token_b_mint;
        let seeds = &[
            b"pool",
            token_a_mint.as_ref(),
            token_b_mint.as_ref(),
            &[pool.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts_out = Transfer {
            from: ctx.accounts.token_a_vault.to_account_info(),
            to: ctx.accounts.user_token_a.to_account_info(),
            authority: pool.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts_out,
                signer,
            ),
            amount_out,
        )?;

        // Update reserves
        pool.reserve_b = pool
            .reserve_b
            .checked_add(amount_in)
            .ok_or(SwapError::MathOverflow)?;
        pool.reserve_a = pool
            .reserve_a
            .checked_sub(amount_out)
            .ok_or(SwapError::MathOverflow)?;

        emit!(Swapped {
            pool: pool.key(),
            user: ctx.accounts.user.key(),
            token_in: pool.token_b_mint,
            token_out: pool.token_a_mint,
            amount_in,
            amount_out,
            fee,
        });

        Ok(())
    }

    /// Get quote for swap (view function - doesn't modify state)
    pub fn get_swap_quote(ctx: Context<GetQuote>, amount_in: u64, is_a_to_b: bool) -> Result<()> {
        let pool = &ctx.accounts.pool;

        let (reserve_in, reserve_out) = if is_a_to_b {
            (pool.reserve_a, pool.reserve_b)
        } else {
            (pool.reserve_b, pool.reserve_a)
        };

        let amount_out =
            calculate_swap_output(amount_in, reserve_in, reserve_out, pool.swap_fee_bps)?;
        let fee = calculate_fee(amount_in, pool.swap_fee_bps)?;
        let price_impact = calculate_price_impact(amount_in, reserve_in)?;

        emit!(SwapQuote {
            pool: pool.key(),
            amount_in,
            amount_out,
            fee,
            price_impact_bps: price_impact,
        });

        Ok(())
    }

    /// Admin: Withdraw collected protocol fees
    pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        let fees_a = pool.collected_fees_a;
        let fees_b = pool.collected_fees_b;

        require!(fees_a > 0 || fees_b > 0, SwapError::NoFeesToWithdraw);

        let token_a_mint = pool.token_a_mint;
        let token_b_mint = pool.token_b_mint;
        let seeds = &[
            b"pool",
            token_a_mint.as_ref(),
            token_b_mint.as_ref(),
            &[pool.bump],
        ];
        let signer = &[&seeds[..]];

        if fees_a > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.token_a_vault.to_account_info(),
                to: ctx.accounts.fee_receiver_a.to_account_info(),
                authority: pool.to_account_info(),
            };
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    cpi_accounts,
                    signer,
                ),
                fees_a,
            )?;
            pool.collected_fees_a = 0;
        }

        if fees_b > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.token_b_vault.to_account_info(),
                to: ctx.accounts.fee_receiver_b.to_account_info(),
                authority: pool.to_account_info(),
            };
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    cpi_accounts,
                    signer,
                ),
                fees_b,
            )?;
            pool.collected_fees_b = 0;
        }

        emit!(FeesWithdrawn {
            pool: pool.key(),
            amount_a: fees_a,
            amount_b: fees_b,
        });

        Ok(())
    }
}

// ============ Helper Functions ============

/// Calculate swap output using constant product formula: x * y = k
/// output = (reserve_out * amount_in * (10000 - fee_bps)) / (reserve_in * 10000 + amount_in * (10000 - fee_bps))
fn calculate_swap_output(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_bps: u64,
) -> Result<u64> {
    let amount_in_with_fee = (amount_in as u128)
        .checked_mul((10000 - fee_bps) as u128)
        .ok_or(SwapError::MathOverflow)?;

    let numerator = amount_in_with_fee
        .checked_mul(reserve_out as u128)
        .ok_or(SwapError::MathOverflow)?;

    let denominator = (reserve_in as u128)
        .checked_mul(10000)
        .ok_or(SwapError::MathOverflow)?
        .checked_add(amount_in_with_fee)
        .ok_or(SwapError::MathOverflow)?;

    let output = numerator
        .checked_div(denominator)
        .ok_or(SwapError::MathOverflow)? as u64;

    Ok(output)
}

fn calculate_fee(amount: u64, fee_bps: u64) -> Result<u64> {
    Ok((amount as u128)
        .checked_mul(fee_bps as u128)
        .ok_or(SwapError::MathOverflow)?
        .checked_div(10000)
        .ok_or(SwapError::MathOverflow)? as u64)
}

fn calculate_price_impact(amount_in: u64, reserve_in: u64) -> Result<u64> {
    // Price impact in basis points
    Ok((amount_in as u128)
        .checked_mul(10000)
        .ok_or(SwapError::MathOverflow)?
        .checked_div(reserve_in as u128)
        .ok_or(SwapError::MathOverflow)? as u64)
}

/// Integer square root helper
trait IntegerSqrt {
    fn integer_sqrt(self) -> Self;
}

impl IntegerSqrt for u128 {
    fn integer_sqrt(self) -> Self {
        if self == 0 {
            return 0;
        }
        let mut x = self;
        let mut y = x.div_ceil(2);
        while y < x {
            x = y;
            y = (x + self / x) / 2;
        }
        x
    }
}

// ============ Account Structs ============

#[derive(Accounts)]
#[instruction(pool_bump: u8)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + LiquidityPool::INIT_SPACE,
        seeds = [b"pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
    )]
    pub pool: Account<'info, LiquidityPool>,

    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        token::mint = token_a_mint,
        token::authority = pool,
    )]
    pub token_a_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        token::mint = token_b_mint,
        token::authority = pool,
    )]
    pub token_b_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = pool,
    )]
    pub lp_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        constraint = user_token_a.owner == user.key(),
        constraint = user_token_a.mint == pool.token_a_mint,
    )]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_b.owner == user.key(),
        constraint = user_token_b.mint == pool.token_b_mint,
    )]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = lp_mint,
        associated_token::authority = user,
    )]
    pub user_lp_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = token_a_vault.key() == pool.token_a_vault,
    )]
    pub token_a_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = token_b_vault.key() == pool.token_b_vault,
    )]
    pub token_b_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = lp_mint.key() == pool.lp_mint,
    )]
    pub lp_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        constraint = user_token_a.owner == user.key(),
        constraint = user_token_a.mint == pool.token_a_mint,
    )]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_b.owner == user.key(),
        constraint = user_token_b.mint == pool.token_b_mint,
    )]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_lp_account.owner == user.key(),
        constraint = user_lp_account.mint == pool.lp_mint,
    )]
    pub user_lp_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = token_a_vault.key() == pool.token_a_vault,
    )]
    pub token_a_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = token_b_vault.key() == pool.token_b_vault,
    )]
    pub token_b_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = lp_mint.key() == pool.lp_mint,
    )]
    pub lp_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        constraint = user_token_a.owner == user.key(),
        constraint = user_token_a.mint == pool.token_a_mint,
    )]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_b.owner == user.key(),
        constraint = user_token_b.mint == pool.token_b_mint,
    )]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = token_a_vault.key() == pool.token_a_vault,
    )]
    pub token_a_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = token_b_vault.key() == pool.token_b_vault,
    )]
    pub token_b_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct GetQuote<'info> {
    pub pool: Account<'info, LiquidityPool>,
}

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(
        constraint = authority.key() == pool.authority,
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        constraint = token_a_vault.key() == pool.token_a_vault,
    )]
    pub token_a_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = token_b_vault.key() == pool.token_b_vault,
    )]
    pub token_b_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = fee_receiver_a.mint == pool.token_a_mint,
    )]
    pub fee_receiver_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = fee_receiver_b.mint == pool.token_b_mint,
    )]
    pub fee_receiver_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

// ============ State Accounts ============

#[account]
#[derive(InitSpace)]
pub struct LiquidityPool {
    pub authority: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub lp_mint: Pubkey,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub total_lp_supply: u64,
    pub swap_fee_bps: u64,
    pub protocol_fee_bps: u64,
    pub collected_fees_a: u64,
    pub collected_fees_b: u64,
    pub bump: u8,
    pub is_active: bool,
    pub created_at: i64,
}

// ============ Events ============

#[event]
pub struct PoolInitialized {
    pub pool: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub lp_mint: Pubkey,
}

#[event]
pub struct LiquidityAdded {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub lp_tokens: u64,
}

#[event]
pub struct LiquidityRemoved {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub lp_tokens: u64,
}

#[event]
pub struct Swapped {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub token_in: Pubkey,
    pub token_out: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee: u64,
}

#[event]
pub struct SwapQuote {
    pub pool: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee: u64,
    pub price_impact_bps: u64,
}

#[event]
pub struct FeesWithdrawn {
    pub pool: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
}

// ============ Errors ============

#[error_code]
pub enum SwapError {
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Pool is not active")]
    PoolInactive,
    #[msg("Insufficient liquidity in pool")]
    InsufficientLiquidity,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("No fees to withdraw")]
    NoFeesToWithdraw,
    #[msg("Math overflow")]
    MathOverflow,
}
