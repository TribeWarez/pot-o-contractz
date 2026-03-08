use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

// Module declarations
pub mod events;
pub mod services;

// Re-export services for use in instructions
use events::{
    Deposited, EscrowCancelled, EscrowCreated, EscrowReleased, LockExtended, TreasuryInitialized,
    VaultCreated, Withdrawn,
};

declare_id!("HmWGA3JAF6basxGCvvGNHAdTBE3qCPhJCeFJAd7r5ra9");

/// Tribewarez Vault Program
/// Secure escrow and vault functionality for PTtC tokens.
/// Supports deposits, withdrawals, time-locked vaults, and multi-sig operations.
///
/// v0.2.0 includes tensor network support for dynamic locktime reduction
/// based on entropy and coherence-aware access control.

#[program]
pub mod tribewarez_vault {
    use super::*;

    /// Initialize the main vault treasury
    pub fn initialize_treasury(ctx: Context<InitializeTreasury>, treasury_bump: u8) -> Result<()> {
        let treasury = &mut ctx.accounts.treasury;

        treasury.authority = ctx.accounts.authority.key();
        treasury.token_mint = ctx.accounts.token_mint.key();
        treasury.vault_token_account = ctx.accounts.vault_token_account.key();
        treasury.total_deposited = 0;
        treasury.total_vaults = 0;
        treasury.bump = treasury_bump;
        treasury.is_active = true;
        treasury.created_at = Clock::get()?.unix_timestamp;

        emit!(TreasuryInitialized {
            treasury: treasury.key(),
            authority: treasury.authority,
            token_mint: treasury.token_mint,
        });

        Ok(())
    }

    /// Create a personal vault for a user
    pub fn create_vault(
        ctx: Context<CreateVault>,
        vault_name: String,
        lock_until: i64, // Unix timestamp, 0 for no lock
    ) -> Result<()> {
        require!(vault_name.len() <= 32, VaultError::NameTooLong);

        let vault = &mut ctx.accounts.user_vault;
        let treasury = &mut ctx.accounts.treasury;

        vault.owner = ctx.accounts.user.key();
        vault.treasury = treasury.key();
        vault.name = vault_name.clone();
        vault.balance = 0;
        vault.lock_until = lock_until;
        vault.created_at = Clock::get()?.unix_timestamp;
        vault.last_activity = Clock::get()?.unix_timestamp;
        vault.is_locked = lock_until > 0;
        vault.total_deposited = 0;
        vault.total_withdrawn = 0;

        treasury.total_vaults = treasury
            .total_vaults
            .checked_add(1)
            .ok_or(VaultError::MathOverflow)?;

        emit!(VaultCreated {
            vault: vault.key(),
            owner: vault.owner,
            name: vault_name,
            lock_until,
        });

        Ok(())
    }

    /// Deposit tokens into a personal vault
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmount);
        require!(
            ctx.accounts.treasury.is_active,
            VaultError::TreasuryInactive
        );

        let vault = &mut ctx.accounts.user_vault;
        let treasury = &mut ctx.accounts.treasury;

        // Transfer tokens from user to treasury vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Update vault state
        vault.balance = vault
            .balance
            .checked_add(amount)
            .ok_or(VaultError::MathOverflow)?;
        vault.total_deposited = vault
            .total_deposited
            .checked_add(amount)
            .ok_or(VaultError::MathOverflow)?;
        vault.last_activity = Clock::get()?.unix_timestamp;

        // Update treasury totals
        treasury.total_deposited = treasury
            .total_deposited
            .checked_add(amount)
            .ok_or(VaultError::MathOverflow)?;

        emit!(Deposited {
            vault: vault.key(),
            user: ctx.accounts.user.key(),
            amount,
            new_balance: vault.balance,
        });

        Ok(())
    }

    /// Withdraw tokens from a personal vault
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmount);

        let clock = Clock::get()?;
        let vault = &mut ctx.accounts.user_vault;
        let treasury = &ctx.accounts.treasury;

        require!(vault.balance >= amount, VaultError::InsufficientBalance);
        require!(
            !vault.is_locked || clock.unix_timestamp >= vault.lock_until,
            VaultError::VaultLocked
        );

        // Transfer tokens from treasury vault to user using PDA signer
        let token_mint = treasury.token_mint;
        let seeds = &[b"treasury", token_mint.as_ref(), &[treasury.bump]];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.treasury.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );
        token::transfer(cpi_ctx, amount)?;

        // Update vault state
        vault.balance = vault
            .balance
            .checked_sub(amount)
            .ok_or(VaultError::MathOverflow)?;
        vault.total_withdrawn = vault
            .total_withdrawn
            .checked_add(amount)
            .ok_or(VaultError::MathOverflow)?;
        vault.last_activity = clock.unix_timestamp;

        emit!(Withdrawn {
            vault: vault.key(),
            user: ctx.accounts.user.key(),
            amount,
            new_balance: vault.balance,
        });

        Ok(())
    }

    /// Create a time-locked escrow between two parties
    pub fn create_escrow(
        ctx: Context<CreateEscrow>,
        amount: u64,
        release_time: i64,
        escrow_bump: u8,
    ) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmount);
        require!(
            release_time > Clock::get()?.unix_timestamp,
            VaultError::InvalidReleaseTime
        );

        let escrow = &mut ctx.accounts.escrow;

        // Transfer tokens to escrow account
        let cpi_accounts = Transfer {
            from: ctx.accounts.depositor_token_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.depositor.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        escrow.depositor = ctx.accounts.depositor.key();
        escrow.beneficiary = ctx.accounts.beneficiary.key();
        escrow.token_mint = ctx.accounts.token_mint.key();
        escrow.escrow_token_account = ctx.accounts.escrow_token_account.key();
        escrow.amount = amount;
        escrow.release_time = release_time;
        escrow.created_at = Clock::get()?.unix_timestamp;
        escrow.is_released = false;
        escrow.is_cancelled = false;
        escrow.bump = escrow_bump;

        emit!(EscrowCreated {
            escrow: escrow.key(),
            depositor: escrow.depositor,
            beneficiary: escrow.beneficiary,
            amount,
            release_time,
        });

        Ok(())
    }

    /// Release escrow funds to beneficiary (after release time)
    pub fn release_escrow(ctx: Context<ReleaseEscrow>) -> Result<()> {
        let clock = Clock::get()?;
        let escrow = &mut ctx.accounts.escrow;

        require!(!escrow.is_released, VaultError::AlreadyReleased);
        require!(!escrow.is_cancelled, VaultError::EscrowCancelled);
        require!(
            clock.unix_timestamp >= escrow.release_time,
            VaultError::EscrowNotReady
        );

        // Transfer tokens to beneficiary using PDA signer
        let depositor = escrow.depositor;
        let beneficiary = escrow.beneficiary;
        let seeds = &[
            b"escrow",
            depositor.as_ref(),
            beneficiary.as_ref(),
            &[escrow.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.beneficiary_token_account.to_account_info(),
            authority: escrow.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );
        token::transfer(cpi_ctx, escrow.amount)?;

        escrow.is_released = true;

        emit!(EscrowReleased {
            escrow: escrow.key(),
            beneficiary: escrow.beneficiary,
            amount: escrow.amount,
        });

        Ok(())
    }

    /// Cancel escrow (only depositor can cancel before release time)
    pub fn cancel_escrow(ctx: Context<CancelEscrow>) -> Result<()> {
        let clock = Clock::get()?;
        let escrow = &mut ctx.accounts.escrow;

        require!(!escrow.is_released, VaultError::AlreadyReleased);
        require!(!escrow.is_cancelled, VaultError::EscrowCancelled);
        require!(
            clock.unix_timestamp < escrow.release_time,
            VaultError::CannotCancelAfterRelease
        );

        // Return tokens to depositor using PDA signer
        let depositor = escrow.depositor;
        let beneficiary = escrow.beneficiary;
        let seeds = &[
            b"escrow",
            depositor.as_ref(),
            beneficiary.as_ref(),
            &[escrow.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.depositor_token_account.to_account_info(),
            authority: escrow.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );
        token::transfer(cpi_ctx, escrow.amount)?;

        escrow.is_cancelled = true;

        emit!(EscrowCancelled {
            escrow: escrow.key(),
            depositor: escrow.depositor,
            amount: escrow.amount,
        });

        Ok(())
    }

    /// Update vault lock time (extend lock)
    pub fn extend_lock(ctx: Context<UpdateVault>, new_lock_until: i64) -> Result<()> {
        let vault = &mut ctx.accounts.user_vault;

        require!(
            new_lock_until > vault.lock_until,
            VaultError::CannotReduceLock
        );

        vault.lock_until = new_lock_until;
        vault.is_locked = true;

        emit!(LockExtended {
            vault: vault.key(),
            owner: vault.owner,
            new_lock_until,
        });

        Ok(())
    }
}

// ============ Account Structs ============

#[derive(Accounts)]
#[instruction(treasury_bump: u8)]
pub struct InitializeTreasury<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + Treasury::INIT_SPACE,
        seeds = [b"treasury", token_mint.key().as_ref()],
        bump,
    )]
    pub treasury: Account<'info, Treasury>,

    pub token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        token::mint = token_mint,
        token::authority = treasury,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(vault_name: String)]
pub struct CreateVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"treasury", treasury.token_mint.as_ref()],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,

    #[account(
        init,
        payer = user,
        space = 8 + UserVault::INIT_SPACE,
        seeds = [b"user_vault", treasury.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub user_vault: Account<'info, UserVault>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"treasury", treasury.token_mint.as_ref()],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,

    #[account(
        mut,
        seeds = [b"user_vault", treasury.key().as_ref(), user.key().as_ref()],
        bump,
        constraint = user_vault.owner == user.key(),
    )]
    pub user_vault: Account<'info, UserVault>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
        constraint = user_token_account.mint == treasury.token_mint,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_token_account.key() == treasury.vault_token_account,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"treasury", treasury.token_mint.as_ref()],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,

    #[account(
        mut,
        seeds = [b"user_vault", treasury.key().as_ref(), user.key().as_ref()],
        bump,
        constraint = user_vault.owner == user.key(),
    )]
    pub user_vault: Account<'info, UserVault>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
        constraint = user_token_account.mint == treasury.token_mint,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_token_account.key() == treasury.vault_token_account,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(amount: u64, release_time: i64, escrow_bump: u8)]
pub struct CreateEscrow<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    /// CHECK: Beneficiary doesn't need to sign for escrow creation
    pub beneficiary: AccountInfo<'info>,

    pub token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = depositor,
        space = 8 + Escrow::INIT_SPACE,
        seeds = [b"escrow", depositor.key().as_ref(), beneficiary.key().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        init,
        payer = depositor,
        token::mint = token_mint,
        token::authority = escrow,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = depositor_token_account.owner == depositor.key(),
        constraint = depositor_token_account.mint == token_mint.key(),
    )]
    pub depositor_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ReleaseEscrow<'info> {
    /// CHECK: Anyone can trigger release after time passes
    pub caller: Signer<'info>,

    #[account(
        mut,
        seeds = [b"escrow", escrow.depositor.as_ref(), escrow.beneficiary.as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        constraint = escrow_token_account.key() == escrow.escrow_token_account,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = beneficiary_token_account.owner == escrow.beneficiary,
        constraint = beneficiary_token_account.mint == escrow.token_mint,
    )]
    pub beneficiary_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelEscrow<'info> {
    #[account(
        constraint = depositor.key() == escrow.depositor,
    )]
    pub depositor: Signer<'info>,

    #[account(
        mut,
        seeds = [b"escrow", escrow.depositor.as_ref(), escrow.beneficiary.as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        constraint = escrow_token_account.key() == escrow.escrow_token_account,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = depositor_token_account.owner == escrow.depositor,
        constraint = depositor_token_account.mint == escrow.token_mint,
    )]
    pub depositor_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateVault<'info> {
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = user_vault.owner == user.key(),
    )]
    pub user_vault: Account<'info, UserVault>,
}

// ============ State Accounts ============

#[account]
#[derive(InitSpace)]
pub struct Treasury {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub vault_token_account: Pubkey,
    pub total_deposited: u64,
    pub total_vaults: u64,
    pub bump: u8,
    pub is_active: bool,
    pub created_at: i64,
}

#[account]
#[derive(InitSpace)]
pub struct UserVault {
    pub owner: Pubkey,
    pub treasury: Pubkey,
    #[max_len(32)]
    pub name: String,
    pub balance: u64,
    pub lock_until: i64,
    pub created_at: i64,
    pub last_activity: i64,
    pub is_locked: bool,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
}

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub depositor: Pubkey,
    pub beneficiary: Pubkey,
    pub token_mint: Pubkey,
    pub escrow_token_account: Pubkey,
    pub amount: u64,
    pub release_time: i64,
    pub created_at: i64,
    pub is_released: bool,
    pub is_cancelled: bool,
    pub bump: u8,
}

// ============ Errors ============

#[error_code]
pub enum VaultError {
    #[msg("Vault name too long (max 32 characters)")]
    NameTooLong,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Treasury is not active")]
    TreasuryInactive,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Vault is locked")]
    VaultLocked,
    #[msg("Invalid release time")]
    InvalidReleaseTime,
    #[msg("Escrow already released")]
    AlreadyReleased,
    #[msg("Escrow was cancelled")]
    EscrowCancelled,
    #[msg("Escrow not ready for release")]
    EscrowNotReady,
    #[msg("Cannot cancel escrow after release time")]
    CannotCancelAfterRelease,
    #[msg("Cannot reduce lock time")]
    CannotReduceLock,
    #[msg("Math overflow")]
    MathOverflow,
}
