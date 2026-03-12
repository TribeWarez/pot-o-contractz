use anchor_lang::prelude::*;

/// Bridge vault account that manages collateral and wrapped token inventory
#[account]
pub struct BridgeVault {
    /// Authority that can manage the vault
    pub vault_authority: Pubkey,

    /// Token A (typically a wrapped token like NMTC)
    pub token_a: Pubkey,

    /// Token B (typically a native token like AUMCOIN)
    pub token_b: Pubkey,

    /// Fee in basis points (100 = 1%)
    pub fee_bps: u16,

    /// Whether the bridge is paused
    pub is_paused: bool,

    /// Total collateral held (in token A)
    pub collateral_balance: u64,

    /// Total wrapped tokens issued (in token B)
    pub wrapped_supply: u64,

    /// Collected fees (in token A)
    pub collected_fees: u64,

    /// Bump seed for vault
    pub bump: u8,
}

impl BridgeVault {
    pub const LEN: usize = 32 + 32 + 32 + 2 + 1 + 8 + 8 + 8 + 1;

    /// Calculate fee on amount (in basis points)
    /// Returns (fee_amount, amount_after_fee)
    pub fn calculate_fee(&self, amount: u64) -> Result<(u64, u64)> {
        let fee = amount
            .checked_mul(self.fee_bps as u64)
            .and_then(|v| v.checked_div(10000))
            .ok_or(error!(crate::errors::BridgeError::ArithmeticOverflow))?;

        let after_fee = amount
            .checked_sub(fee)
            .ok_or(error!(crate::errors::BridgeError::ArithmeticOverflow))?;

        Ok((fee, after_fee))
    }

    /// Validate collateral backing (must be 1:1 or better)
    pub fn validate_collateral(&self) -> Result<()> {
        if self.collateral_balance < self.wrapped_supply {
            Err(error!(crate::errors::BridgeError::InvalidCollateralRatio))
        } else {
            Ok(())
        }
    }

    /// Deposit collateral to vault
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        self.collateral_balance = self
            .collateral_balance
            .checked_add(amount)
            .ok_or(error!(crate::errors::BridgeError::ArithmeticOverflow))?;
        Ok(())
    }

    /// Withdraw collateral from vault, ensuring 1:1 backing
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        if self.collateral_balance < amount {
            return Err(error!(crate::errors::BridgeError::InsufficientVaultBalance));
        }
        self.collateral_balance = self
            .collateral_balance
            .checked_sub(amount)
            .ok_or(error!(crate::errors::BridgeError::ArithmeticOverflow))?;
        // Ensure still backed
        self.validate_collateral()?;
        Ok(())
    }

    /// Mint wrapped tokens (increases wrapped supply)
    pub fn mint_wrapped(&mut self, amount: u64) -> Result<()> {
        self.wrapped_supply = self
            .wrapped_supply
            .checked_add(amount)
            .ok_or(error!(crate::errors::BridgeError::ArithmeticOverflow))?;
        Ok(())
    }

    /// Burn wrapped tokens (decreases wrapped supply)
    pub fn burn_wrapped(&mut self, amount: u64) -> Result<()> {
        if self.wrapped_supply < amount {
            return Err(error!(crate::errors::BridgeError::InsufficientBalance));
        }
        self.wrapped_supply = self
            .wrapped_supply
            .checked_sub(amount)
            .ok_or(error!(crate::errors::BridgeError::ArithmeticOverflow))?;
        Ok(())
    }

    /// Collect fees
    pub fn collect_fee(&mut self, fee_amount: u64) -> Result<()> {
        self.collected_fees = self
            .collected_fees
            .checked_add(fee_amount)
            .ok_or(error!(crate::errors::BridgeError::ArithmeticOverflow))?;
        Ok(())
    }

    /// Withdraw collected fees
    pub fn withdraw_fees(&mut self, amount: u64) -> Result<()> {
        if self.collected_fees < amount {
            return Err(error!(crate::errors::BridgeError::InsufficientVaultBalance));
        }
        self.collected_fees = self
            .collected_fees
            .checked_sub(amount)
            .ok_or(error!(crate::errors::BridgeError::ArithmeticOverflow))?;
        Ok(())
    }

    /// Check if vault is operational
    pub fn is_operational(&self) -> bool {
        !self.is_paused
    }

    /// Pause vault operations
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// Resume vault operations
    pub fn resume(&mut self) {
        self.is_paused = false;
    }
}

/// Wrapped token metadata
#[account]
pub struct WrappedToken {
    /// Original token address
    pub original_token: Pubkey,

    /// Wrapped token mint
    pub wrapped_mint: Pubkey,

    /// Bridge vault that backs this wrapped token
    pub vault: Pubkey,

    /// Total amount wrapped
    pub total_wrapped: u64,

    /// Decimals of the original token
    pub decimals: u8,

    /// Token name
    pub name: String,

    /// Token symbol
    pub symbol: String,

    /// Bump seed
    pub bump: u8,
}

impl WrappedToken {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 1 + 32 + 16 + 1;
}
