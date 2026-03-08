//! # tribewarez-staking
//!
//! PTtC (Power-to-Trust-to-Collaborate) Staking Program for Tribewarez DeFi.
//!
//! This crate implements a staking program that allows users to stake PTtC tokens and earn rewards
//! over time. It supports flexible staking strategies with v0.2.0 introducing tensor network entanglement
//! for cooperative staking with entropy-based unlock probabilities and efficiency bonuses.
//!
//! ## Core Features
//!
//! - **Token Staking**: Secure staking of PTtC tokens with time-based rewards
//! - **Reward Distribution**: Distribute rewards to stakers based on stake amount and duration
//! - **Staking Pools**: Multiple pools with different reward rates and lock durations
//! - **Tensor Entanglement**: v0.2.0 feature enabling cooperative staking with quantum-like entanglement
//! - **Unlock Mechanics**: Entropy-based unlock probabilities for enhanced security
//!
//! ## Key Instructions
//!
//! - `initialize_pool`: Create a new staking pool with specific parameters
//! - `stake`: Stake tokens into a pool
//! - `unstake`: Withdraw staked tokens (subject to lock duration)
//! - `claim_rewards`: Claim accumulated rewards
//! - `entangle_stakes`: Link stakes in tensor network for cooperative rewards (v0.2.0)
//!
//! ## Events
//!
//! This program emits events for stake deposits, withdrawals, reward claims, and pool configuration changes.
//! See the [`events`] module for detailed event documentation.
//!
//! ## Security Considerations
//!
//! - Staking locks are enforced via timestamp checks
//! - Token transfers use SPL Token program via CPI for security
//! - Admin-only operations are protected via Anchor's access control

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Transfer};

// Module declarations
pub mod events;
pub mod services;

// Re-export services for use in instructions
use events::{PoolInitialized, PoolUpdated, RewardsClaimed, Staked, Unstaked};

declare_id!("Go2BZRhNLoaVni3QunrKPAXYdHtwZtTXuVspxpdAeDS8");

/// Tribewarez Staking Program
/// Allows users to stake PTtC tokens and earn rewards over time.
/// Using native SPL token CPI calls for compatibility.
///
/// v0.2.0 includes tensor network entanglement for cooperative staking
/// with entropy-based unlock probabilities and efficiency bonuses.

#[program]
pub mod tribewarez_staking {
    use super::*;

    /// Initialize a new staking pool for a specific token mint (PTtC)
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        reward_rate: u64,   // Rewards per second per token staked (in basis points)
        lock_duration: i64, // Minimum lock duration in seconds
    ) -> Result<()> {
        let pool = &mut ctx.accounts.staking_pool;

        pool.authority = ctx.accounts.authority.key();
        pool.token_mint = ctx.accounts.token_mint.key();
        pool.reward_mint = ctx.accounts.reward_mint.key();
        pool.pool_token_account = ctx.accounts.pool_token_account.key();
        pool.reward_token_account = ctx.accounts.reward_token_account.key();
        pool.reward_rate = reward_rate;
        pool.lock_duration = lock_duration;
        pool.total_staked = 0;
        pool.total_rewards_distributed = 0;
        pool.bump = ctx.bumps.staking_pool;
        pool.is_active = true;
        pool.created_at = Clock::get()?.unix_timestamp;

        emit!(PoolInitialized {
            pool: pool.key(),
            authority: pool.authority,
            token_mint: pool.token_mint,
            reward_rate,
            lock_duration,
        });

        Ok(())
    }

    /// Stake PTtC tokens into the pool
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        require!(amount > 0, StakingError::InvalidAmount);
        require!(
            ctx.accounts.staking_pool.is_active,
            StakingError::PoolInactive
        );

        let clock = Clock::get()?;
        let stake_account = &mut ctx.accounts.stake_account;
        let pool = &mut ctx.accounts.staking_pool;

        // Calculate pending rewards before updating stake
        if stake_account.amount > 0 {
            let pending = calculate_rewards(
                stake_account.amount,
                stake_account.last_reward_time,
                clock.unix_timestamp,
                pool.reward_rate,
            )?;
            stake_account.pending_rewards = stake_account
                .pending_rewards
                .checked_add(pending)
                .ok_or(StakingError::MathOverflow)?;
        }

        // Transfer tokens from user to pool using anchor-spl
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.pool_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
            amount,
        )?;

        // Update stake account
        stake_account.owner = ctx.accounts.user.key();
        stake_account.pool = pool.key();
        stake_account.amount = stake_account
            .amount
            .checked_add(amount)
            .ok_or(StakingError::MathOverflow)?;
        stake_account.stake_time = clock.unix_timestamp;
        stake_account.last_reward_time = clock.unix_timestamp;
        stake_account.unlock_time = clock.unix_timestamp + pool.lock_duration;

        // Update pool totals
        pool.total_staked = pool
            .total_staked
            .checked_add(amount)
            .ok_or(StakingError::MathOverflow)?;

        emit!(Staked {
            user: ctx.accounts.user.key(),
            pool: pool.key(),
            amount,
            total_staked: stake_account.amount,
            unlock_time: stake_account.unlock_time,
        });

        Ok(())
    }

    /// Unstake tokens from the pool
    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        require!(amount > 0, StakingError::InvalidAmount);

        let clock = Clock::get()?;
        // Capture AccountInfo before the mutable borrow below: the PDA-signed CPI needs
        // staking_pool's AccountInfo as authority, but the state update also needs &mut.
        let pool_account_info = ctx.accounts.staking_pool.to_account_info();
        let stake_account = &mut ctx.accounts.stake_account;
        let pool = &mut ctx.accounts.staking_pool;

        require!(
            stake_account.amount >= amount,
            StakingError::InsufficientStake
        );
        require!(
            clock.unix_timestamp >= stake_account.unlock_time,
            StakingError::StillLocked
        );

        // Calculate and add pending rewards
        let pending = calculate_rewards(
            stake_account.amount,
            stake_account.last_reward_time,
            clock.unix_timestamp,
            pool.reward_rate,
        )?;
        stake_account.pending_rewards = stake_account
            .pending_rewards
            .checked_add(pending)
            .ok_or(StakingError::MathOverflow)?;
        stake_account.last_reward_time = clock.unix_timestamp;

        // Transfer tokens back to user using PDA signer
        let token_mint = pool.token_mint;
        let seeds = &[b"staking_pool", token_mint.as_ref(), &[pool.bump]];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: pool_account_info,
        };
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer,
            ),
            amount,
        )?;

        // Update stake account
        stake_account.amount = stake_account
            .amount
            .checked_sub(amount)
            .ok_or(StakingError::MathOverflow)?;

        // Update pool totals
        pool.total_staked = pool
            .total_staked
            .checked_sub(amount)
            .ok_or(StakingError::MathOverflow)?;

        emit!(Unstaked {
            user: ctx.accounts.user.key(),
            pool: pool.key(),
            amount,
            remaining_stake: stake_account.amount,
        });

        Ok(())
    }

    /// Claim accumulated rewards
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let clock = Clock::get()?;
        // Capture AccountInfo before the mutable borrow below: the PDA-signed CPI needs
        // staking_pool's AccountInfo as authority, but the state update also needs &mut.
        let pool_account_info = ctx.accounts.staking_pool.to_account_info();
        let stake_account = &mut ctx.accounts.stake_account;
        let pool = &mut ctx.accounts.staking_pool;

        // Calculate current pending rewards
        let pending = calculate_rewards(
            stake_account.amount,
            stake_account.last_reward_time,
            clock.unix_timestamp,
            pool.reward_rate,
        )?;

        let total_rewards = stake_account
            .pending_rewards
            .checked_add(pending)
            .ok_or(StakingError::MathOverflow)?;

        require!(total_rewards > 0, StakingError::NoRewardsToClaim);

        // Transfer rewards to user using PDA signer
        let token_mint = pool.token_mint;
        let seeds = &[b"staking_pool", token_mint.as_ref(), &[pool.bump]];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.reward_token_account.to_account_info(),
            to: ctx.accounts.user_reward_account.to_account_info(),
            authority: pool_account_info,
        };
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer,
            ),
            total_rewards,
        )?;

        // Update state
        stake_account.pending_rewards = 0;
        stake_account.last_reward_time = clock.unix_timestamp;
        stake_account.total_rewards_claimed = stake_account
            .total_rewards_claimed
            .checked_add(total_rewards)
            .ok_or(StakingError::MathOverflow)?;

        pool.total_rewards_distributed = pool
            .total_rewards_distributed
            .checked_add(total_rewards)
            .ok_or(StakingError::MathOverflow)?;

        emit!(RewardsClaimed {
            user: ctx.accounts.user.key(),
            pool: pool.key(),
            amount: total_rewards,
            total_claimed: stake_account.total_rewards_claimed,
        });

        Ok(())
    }

    /// Admin: Update pool parameters
    pub fn update_pool(
        ctx: Context<UpdatePool>,
        new_reward_rate: Option<u64>,
        new_lock_duration: Option<i64>,
        is_active: Option<bool>,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.staking_pool;

        if let Some(rate) = new_reward_rate {
            pool.reward_rate = rate;
        }
        if let Some(duration) = new_lock_duration {
            pool.lock_duration = duration;
        }
        if let Some(active) = is_active {
            pool.is_active = active;
        }

        emit!(PoolUpdated {
            pool: pool.key(),
            reward_rate: pool.reward_rate,
            lock_duration: pool.lock_duration,
            is_active: pool.is_active,
        });

        Ok(())
    }
}

// ============ Helper Functions ============

fn calculate_rewards(
    staked_amount: u64,
    last_reward_time: i64,
    current_time: i64,
    reward_rate: u64,
) -> Result<u64> {
    let time_elapsed = (current_time - last_reward_time) as u64;

    // rewards = (staked_amount * time_elapsed * reward_rate) / 10000 / SECONDS_PER_DAY
    let rewards = (staked_amount as u128)
        .checked_mul(time_elapsed as u128)
        .ok_or(StakingError::MathOverflow)?
        .checked_mul(reward_rate as u128)
        .ok_or(StakingError::MathOverflow)?
        .checked_div(10000 * 86400)
        .ok_or(StakingError::MathOverflow)?;

    Ok(rewards as u64)
}

// ============ Account Structs ============

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + StakingPool::INIT_SPACE,
        seeds = [b"staking_pool", token_mint.key().as_ref()],
        bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    /// CHECK: Token mint account
    pub token_mint: AccountInfo<'info>,
    /// CHECK: Reward mint account
    pub reward_mint: AccountInfo<'info>,

    /// CHECK: Pool's token account for staked tokens
    #[account(mut)]
    pub pool_token_account: AccountInfo<'info>,

    /// CHECK: Pool's reward token account
    #[account(mut)]
    pub reward_token_account: AccountInfo<'info>,

    /// CHECK: SPL Token program
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"staking_pool", staking_pool.token_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + StakeAccount::INIT_SPACE,
        seeds = [b"stake", staking_pool.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    /// CHECK: User's token account
    #[account(mut)]
    pub user_token_account: AccountInfo<'info>,

    /// CHECK: Pool's token account
    #[account(mut)]
    pub pool_token_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"staking_pool", staking_pool.token_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        seeds = [b"stake", staking_pool.key().as_ref(), user.key().as_ref()],
        bump,
        constraint = stake_account.owner == user.key(),
    )]
    pub stake_account: Account<'info, StakeAccount>,

    /// CHECK: User's token account
    #[account(mut)]
    pub user_token_account: AccountInfo<'info>,

    /// CHECK: Pool's token account
    #[account(mut)]
    pub pool_token_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"staking_pool", staking_pool.token_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        seeds = [b"stake", staking_pool.key().as_ref(), user.key().as_ref()],
        bump,
        constraint = stake_account.owner == user.key(),
    )]
    pub stake_account: Account<'info, StakeAccount>,

    /// CHECK: User's reward token account
    #[account(mut)]
    pub user_reward_account: AccountInfo<'info>,

    /// CHECK: Pool's reward token account
    #[account(mut)]
    pub reward_token_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdatePool<'info> {
    #[account(
        constraint = authority.key() == staking_pool.authority,
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"staking_pool", staking_pool.token_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,
}

// ============ State Accounts ============

/// A staking pool that holds staked tokens and distributes rewards.
/// Each pool has its own configuration, token accounts, and reward parameters.
#[account]
#[derive(InitSpace)]
pub struct StakingPool {
    /// Authority/admin who can update pool configuration
    pub authority: Pubkey,
    /// Mint of the token being staked in this pool
    pub token_mint: Pubkey,
    /// Mint of the reward token (may be same as token_mint)
    pub reward_mint: Pubkey,
    /// Pool's token account holding staked tokens
    pub pool_token_account: Pubkey,
    /// Pool's token account holding rewards to distribute
    pub reward_token_account: Pubkey,
    /// Reward amount per unit time (in lamports per slot)
    pub reward_rate: u64,
    /// Minimum lock duration for stakes (in seconds)
    pub lock_duration: i64,
    /// Total tokens currently staked in this pool
    pub total_staked: u64,
    /// Cumulative rewards distributed from this pool
    pub total_rewards_distributed: u64,
    /// PDA bump seed for this account
    pub bump: u8,
    /// Whether this pool is accepting new stakes
    pub is_active: bool,
    /// Timestamp when this pool was created
    pub created_at: i64,

    // --- v0.2.0 Tensor Network Extensions ---
    /// Whether tensor network enhancements are enabled for this pool
    pub tensor_enabled: u8,          // 0 = disabled, 1 = enabled
    /// Maximum entropy target for this pool (1e6 scale)
    pub s_max: u64,                  // Maximum entropy (1e6 scale)
    /// Weight factor for entropy in reward calculations (1e6 scale)
    pub entropy_weight: u64,         // Entropy contribution weight (1e6 scale)
    /// Number of stakes participating in entanglement
    pub total_entangled_stakes: u32, // Number of stakes in entangled pools
    /// Sum of all stake entropy contributions
    pub total_pool_entropy: u64,     // Sum of all stake entropies
    /// Average coherence preservation across all pool members
    pub average_coherence: u64,      // Average coherence of pool members
}


/// Individual stake account tracking one user's stake in a pool.
/// Each staker has one account per pool they're participating in.
#[account]
#[derive(InitSpace)]
pub struct StakeAccount {
    /// User/owner of this stake
    pub owner: Pubkey,
    /// Pool this stake belongs to
    pub pool: Pubkey,
    /// Amount of tokens staked
    pub amount: u64,
    /// Timestamp when the stake was created
    pub stake_time: i64,
    /// Timestamp when the stake can be withdrawn (after lock_duration)
    pub unlock_time: i64,
    /// Last timestamp when rewards were calculated
    pub last_reward_time: i64,
    /// Rewards accumulated but not yet claimed
    pub pending_rewards: u64,
    /// Total rewards claimed from this stake
    pub total_rewards_claimed: u64,

    // --- v0.2.0 Tensor Network Extensions ---
    /// Stake's contribution to network entropy (1e6 scale)
    pub entropy_score: u64,       // Stake's entropy contribution (1e6 scale)
    /// Device's ability to preserve quantum coherence (1e6 scale, 0-1000000)
    pub coherence: u64,           // Device coherence preservation (1e6 scale)
    /// ID of the entanglement pool this stake belongs to (0 = not entangled)
    pub pool_id: u32,             // Entanglement pool assignment (0 = not entangled)
    /// Last timestamp when entropy metrics were calculated
    pub last_entropy_update: i64, // Last slot when entropy was calculated
    /// Probability of early unlock based on entropy (1e6 scale, 0-1000000)
    pub unlock_probability: u64,  // P(early unlock) from entropy (1e6 scale)
    /// Bonus multiplier from coherence contribution (1e6 scale)
    pub coherence_bonus: u64,     // Bonus multiplier from coherence (1e6 scale)
}

// ============ Errors ============

#[error_code]
pub enum StakingError {
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Pool is not active")]
    PoolInactive,
    #[msg("Insufficient stake balance")]
    InsufficientStake,
    #[msg("Tokens are still locked")]
    StillLocked,
    #[msg("No rewards to claim")]
    NoRewardsToClaim,
    #[msg("No stake to withdraw")]
    NoStake,
    #[msg("Math overflow")]
    MathOverflow,
}
