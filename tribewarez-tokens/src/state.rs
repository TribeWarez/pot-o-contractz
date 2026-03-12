//! Token program state structures

use anchor_lang::prelude::*;

/// Represents a token mint on the blockchain
#[account]
#[derive(Default)]
pub struct TokenMint {
    /// Address of the mint authority
    pub mint_authority: Pubkey,
    /// Address of the freeze authority
    pub freeze_authority: Pubkey,
    /// Address of the treasury
    pub treasury_address: Pubkey,
    /// Number of decimal places for tokens
    pub decimals: u8,
    /// Optional supply cap (None = unlimited)
    pub supply_cap: Option<u64>,
    /// Total supply of tokens that have been minted
    pub total_supply: u64,
    /// Total tokens minted (including burned)
    pub total_minted: u64,
    /// Total tokens burned
    pub total_burned: u64,
    /// Optional inflation rate (as fraction, e.g., 0.05 = 5%)
    pub inflation_rate: Option<f64>,
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Token URI (logo, metadata)
    pub uri: Option<String>,
    /// Timestamp of mint creation
    pub created_at: i64,
}

/// Represents a user's token account
#[account]
#[derive(Default)]
pub struct TokenAccount {
    /// Owner of the account
    pub owner: Pubkey,
    /// Mint this account holds tokens from
    pub mint: Pubkey,
    /// Token balance
    pub balance: u64,
    /// Whether the account is frozen
    pub is_frozen: bool,
    /// Optional delegate authority (for approval)
    pub delegate: Option<Pubkey>,
    /// Amount the delegate is allowed to transfer
    pub delegated_amount: u64,
    /// Timestamp of account creation
    pub created_at: i64,
}

impl TokenMint {
    /// Space required for serialization
    pub const SPACE: usize = 8 // discriminator
        + 32 // mint_authority
        + 32 // freeze_authority
        + 32 // treasury_address
        + 1  // decimals
        + 9  // supply_cap (option)
        + 8  // total_supply
        + 8  // total_minted
        + 8  // total_burned
        + 9  // inflation_rate (option)
        + (4 + 64)  // name (string)
        + (4 + 16)  // symbol (string)
        + (1 + 4 + 256)  // uri (option + string)
        + 8; // created_at

    /// Validates mint configuration
    pub fn validate(&self) -> Result<()> {
        require!(
            self.decimals <= 18,
            crate::errors::TokenError::InvalidDecimals
        );
        if let Some(cap) = self.supply_cap {
            require!(
                self.total_supply <= cap,
                crate::errors::TokenError::SupplyCapExceeded
            );
        }
        Ok(())
    }

    /// Check if mint can accommodate additional tokens
    pub fn can_mint(&self, amount: u64) -> bool {
        if let Some(cap) = self.supply_cap {
            self.total_supply.saturating_add(amount) <= cap
        } else {
            true
        }
    }

    /// Check if inflation would exceed cap
    pub fn can_inflate(&self, amount: u64) -> bool {
        self.can_mint(amount)
    }
}

impl TokenAccount {
    /// Space required for serialization
    pub const SPACE: usize = 8 // discriminator
        + 32 // owner
        + 32 // mint
        + 8  // balance
        + 1  // is_frozen
        + (1 + 32)  // delegate (option)
        + 8  // delegated_amount
        + 8; // created_at

    /// Check if account can be used
    pub fn is_usable(&self) -> bool {
        !self.is_frozen
    }

    /// Get available balance for spending (accounting for delegation)
    pub fn get_available_balance(&self) -> u64 {
        self.balance
    }

    /// Check if account can send amount
    pub fn can_transfer(&self, amount: u64) -> bool {
        !self.is_frozen && self.balance >= amount
    }
}
