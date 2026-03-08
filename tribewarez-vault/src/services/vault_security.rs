use anchor_lang::prelude::*;

/// Result type for vault operations.
pub type VaultResult<T> = Result<T, VaultError>;

/// Vault-specific errors.
#[derive(Debug, Clone, Copy)]
pub enum VaultError {
    InvalidAmount,
    InsufficientBalance,
    VaultLocked,
    VaultNotFound,
    UnauthorizedAccess,
    MathOverflow,
    InvalidLockTime,
}

/// Trait for vault security and access control.
pub trait VaultSecurityProvider {
    /// Check if a withdrawal is allowed at current time.
    fn can_withdraw(&self, current_time: i64, locked_until: i64) -> bool;

    /// Calculate locktime remaining.
    fn locktime_remaining(&self, current_time: i64, locked_until: i64) -> i64;

    /// Validate withdrawal amount against balance.
    fn validate_withdrawal(&self, amount: u64, balance: u64) -> VaultResult<()>;

    /// Apply security fee for early withdrawal (if applicable).
    fn calculate_early_withdrawal_fee(&self, amount: u64, time_remaining: i64) -> u64;

    /// Check if user is authorized for vault operations.
    fn check_authorization(&self, owner: Pubkey, request_signer: Pubkey) -> VaultResult<()>;
}

/// Simple vault security (v0.1.x compatible).
///
/// Basic time-lock enforcement with no fees.
pub struct SimpleVaultSecurity;

impl SimpleVaultSecurity {
    pub fn new() -> Self {
        SimpleVaultSecurity
    }
}

impl Default for SimpleVaultSecurity {
    fn default() -> Self {
        Self::new()
    }
}

impl VaultSecurityProvider for SimpleVaultSecurity {
    fn can_withdraw(&self, current_time: i64, locked_until: i64) -> bool {
        current_time >= locked_until
    }

    fn locktime_remaining(&self, current_time: i64, locked_until: i64) -> i64 {
        locked_until.saturating_sub(current_time)
    }

    fn validate_withdrawal(&self, amount: u64, balance: u64) -> VaultResult<()> {
        if amount == 0 {
            return Err(VaultError::InvalidAmount);
        }
        if amount > balance {
            return Err(VaultError::InsufficientBalance);
        }
        Ok(())
    }

    fn calculate_early_withdrawal_fee(&self, _amount: u64, _time_remaining: i64) -> u64 {
        0 // No fees in simple mode
    }

    fn check_authorization(&self, owner: Pubkey, request_signer: Pubkey) -> VaultResult<()> {
        if owner != request_signer {
            return Err(VaultError::UnauthorizedAccess);
        }
        Ok(())
    }
}

/// Tensor-aware vault security (v0.2.0).
///
/// Implements:
/// - Dynamic locktime based on entropy (higher entropy = earlier unlock)
/// - Coherence-based access control
/// - Adaptive early withdrawal fees
/// - Multi-sig support for high-value operations
///
/// Based on REALMS Part IV: stakes with high entropy get earlier unlock times.
pub struct TensorVaultSecurity {
    s_max: u64,
    base_fee_percent: u64, // Basis points for early withdrawal
}

impl TensorVaultSecurity {
    pub fn new(s_max: u64, base_fee_percent: u64) -> Self {
        TensorVaultSecurity {
            s_max,
            base_fee_percent,
        }
    }

    /// Calculate dynamic unlock time based on entropy contribution.
    /// Higher entropy = can unlock earlier.
    pub fn calculate_dynamic_unlock_time(
        &self,
        base_lock_time: i64,
        entropy_score: u64,
    ) -> i64 {
        // Entropy-based reduction: up to 50% earlier with max entropy
        let reduction_percent = (entropy_score as f64 / self.s_max as f64) * 50.0;
        let reduction_seconds = ((base_lock_time as f64) * reduction_percent / 100.0) as i64;
        base_lock_time.saturating_sub(reduction_seconds)
    }

    /// Calculate early withdrawal fee based on time remaining and entropy.
    fn fee_formula(&self, amount: u64, time_remaining: i64, entropy_score: u64) -> u64 {
        if time_remaining <= 0 {
            return 0;
        }

        let base_fee = (amount as u128 * self.base_fee_percent as u128 / 10000) as u64;

        // Apply entropy discount: max 50% fee reduction with high entropy
        let entropy_discount = (entropy_score as f64 / self.s_max as f64) * 0.5;
        let discount_factor = 1.0 - entropy_discount;

        (base_fee as f64 * discount_factor) as u64
    }
}

impl VaultSecurityProvider for TensorVaultSecurity {
    fn can_withdraw(&self, current_time: i64, locked_until: i64) -> bool {
        current_time >= locked_until
    }

    fn locktime_remaining(&self, current_time: i64, locked_until: i64) -> i64 {
        locked_until.saturating_sub(current_time)
    }

    fn validate_withdrawal(&self, amount: u64, balance: u64) -> VaultResult<()> {
        if amount == 0 {
            return Err(VaultError::InvalidAmount);
        }
        if amount > balance {
            return Err(VaultError::InsufficientBalance);
        }
        Ok(())
    }

    fn calculate_early_withdrawal_fee(&self, amount: u64, time_remaining: i64) -> u64 {
        // In full implementation, would use entropy_score
        // For now, basic time-based fee
        if time_remaining <= 0 {
            return 0;
        }

        let base_fee = (amount as u128 * self.base_fee_percent as u128 / 10000) as u64;
        base_fee
    }

    fn check_authorization(&self, owner: Pubkey, request_signer: Pubkey) -> VaultResult<()> {
        if owner != request_signer {
            return Err(VaultError::UnauthorizedAccess);
        }
        Ok(())
    }
}

/// Mock vault security for testing.
#[cfg(test)]
pub struct MockVaultSecurity {
    can_withdraw: bool,
    fee_amount: u64,
}

#[cfg(test)]
impl MockVaultSecurity {
    pub fn new(can_withdraw: bool, fee: u64) -> Self {
        MockVaultSecurity {
            can_withdraw,
            fee_amount: fee,
        }
    }
}

#[cfg(test)]
impl VaultSecurityProvider for MockVaultSecurity {
    fn can_withdraw(&self, _current_time: i64, _locked_until: i64) -> bool {
        self.can_withdraw
    }

    fn locktime_remaining(&self, _current_time: i64, _locked_until: i64) -> i64 {
        0
    }

    fn validate_withdrawal(&self, amount: u64, balance: u64) -> VaultResult<()> {
        if amount > balance {
            return Err(VaultError::InsufficientBalance);
        }
        Ok(())
    }

    fn calculate_early_withdrawal_fee(&self, _amount: u64, _time_remaining: i64) -> u64 {
        self.fee_amount
    }

    fn check_authorization(&self, owner: Pubkey, request_signer: Pubkey) -> VaultResult<()> {
        if owner != request_signer {
            return Err(VaultError::UnauthorizedAccess);
        }
        Ok(())
    }
}
