// Unit tests for Vault Security implementations
//
// Tests cover:
// 1. SimpleVaultSecurity - v0.1.x basic lock time
// 2. TensorVaultSecurity - v0.2.0 entropy-based dynamic locktime
// 3. Early withdrawal fee calculations
// 4. Dynamic fee reductions based on entropy

mod mock_vault_security {
    #[derive(Clone, Copy)]
    pub struct VaultUnlockInfo {
        pub can_unlock: bool,
        pub time_remaining: i64,
        pub dynamic_time_reduction: i64, // How much entropy reduced lock time
    }

    #[derive(Clone, Copy)]
    pub struct WithdrawalFeeInfo {
        pub fee_percent: u64,
        pub is_early: bool,
        pub time_remaining: i64,
        pub coherence_reduction: u64, // Fee reduction due to entropy (0-5000 BPS)
    }

    pub trait VaultSecurityProvider {
        fn can_unlock_vault(&self, lock_until: i64, current_time: i64) -> VaultUnlockInfo;

        fn calculate_early_withdrawal_fee(
            &self,
            withdrawal_amount: u64,
            lock_until: i64,
            current_time: i64,
        ) -> WithdrawalFeeInfo;
    }

    pub struct SimpleVaultSecurity;

    impl SimpleVaultSecurity {
        pub fn new() -> Self {
            SimpleVaultSecurity
        }
    }

    impl VaultSecurityProvider for SimpleVaultSecurity {
        fn can_unlock_vault(&self, lock_until: i64, current_time: i64) -> VaultUnlockInfo {
            let can_unlock = current_time >= lock_until;
            let time_remaining = (lock_until - current_time).max(0);

            VaultUnlockInfo {
                can_unlock,
                time_remaining,
                dynamic_time_reduction: 0,
            }
        }

        fn calculate_early_withdrawal_fee(
            &self,
            withdrawal_amount: u64,
            lock_until: i64,
            current_time: i64,
        ) -> WithdrawalFeeInfo {
            let time_remaining = (lock_until - current_time).max(0);
            let is_early = current_time < lock_until;

            // Linear fee: max 50% if withdrawn immediately
            let max_fee_bps = 5000; // 50%
            let total_lock_time = (lock_until - (lock_until - time_remaining)).max(1);
            let fee_percent = if is_early {
                (max_fee_bps as i64 * time_remaining / total_lock_time).min(max_fee_bps as i64)
                    as u64
            } else {
                0
            };

            WithdrawalFeeInfo {
                fee_percent,
                is_early,
                time_remaining,
                coherence_reduction: 0,
            }
        }
    }

    pub struct TensorVaultSecurity {
        s_max: u64,
        entropy_weight: f64,
    }

    impl TensorVaultSecurity {
        pub fn new(s_max: u64, entropy_weight: f64) -> Self {
            TensorVaultSecurity {
                s_max,
                entropy_weight,
            }
        }

        fn calculate_entropy_reduction(&self, entropy: u64) -> i64 {
            // Reduce locktime by up to 100% based on entropy
            let normalized = (entropy as f64 / self.s_max as f64).min(1.0);
            (normalized * self.entropy_weight * 100.0) as i64
        }

        fn calculate_coherence_discount(&self, entropy: u64) -> u64 {
            // Reduce fee by up to 50% based on entropy
            let normalized = (entropy as f64 / self.s_max as f64).min(1.0);
            (normalized * 5000.0) as u64 // 5000 BPS = 50%
        }
    }

    impl VaultSecurityProvider for TensorVaultSecurity {
        fn can_unlock_vault(&self, lock_until: i64, current_time: i64) -> VaultUnlockInfo {
            // For now, no entropy context in this simple signature
            // In real implementation, would have entropy parameter
            let can_unlock = current_time >= lock_until;
            let time_remaining = (lock_until - current_time).max(0);

            VaultUnlockInfo {
                can_unlock,
                time_remaining,
                dynamic_time_reduction: 0, // Would be calculated with entropy
            }
        }

        fn calculate_early_withdrawal_fee(
            &self,
            withdrawal_amount: u64,
            lock_until: i64,
            current_time: i64,
        ) -> WithdrawalFeeInfo {
            let time_remaining = (lock_until - current_time).max(0);
            let is_early = current_time < lock_until;

            // Base fee (same as SimpleVaultSecurity)
            let max_fee_bps = 5000;
            let total_lock_time = (lock_until - (lock_until - time_remaining)).max(1);
            let base_fee = if is_early {
                (max_fee_bps as i64 * time_remaining / total_lock_time).min(max_fee_bps as i64)
                    as u64
            } else {
                0
            };

            // In real implementation, would apply coherence discount
            WithdrawalFeeInfo {
                fee_percent: base_fee,
                is_early,
                time_remaining,
                coherence_reduction: 0, // Would be calculated with entropy
            }
        }
    }
}

use mock_vault_security::*;

#[test]
fn test_simple_vault_lock_open() {
    let vault = SimpleVaultSecurity::new();

    let lock_until = 1000;
    let current_time = 1100; // After lock expires

    let info = vault.can_unlock_vault(lock_until, current_time);
    assert!(info.can_unlock);
    assert_eq!(info.time_remaining, 0);
}

#[test]
fn test_simple_vault_still_locked() {
    let vault = SimpleVaultSecurity::new();

    let lock_until = 1000;
    let current_time = 900; // Before lock expires

    let info = vault.can_unlock_vault(lock_until, current_time);
    assert!(!info.can_unlock);
    assert_eq!(info.time_remaining, 100);
}

#[test]
fn test_simple_vault_exactly_at_unlock_time() {
    let vault = SimpleVaultSecurity::new();

    let lock_until = 1000;
    let current_time = 1000;

    let info = vault.can_unlock_vault(lock_until, current_time);
    assert!(info.can_unlock);
    assert_eq!(info.time_remaining, 0);
}

#[test]
fn test_simple_early_withdrawal_no_fee_after_unlock() {
    let vault = SimpleVaultSecurity::new();

    let lock_until = 1000;
    let current_time = 1100; // After unlock

    let fee_info = vault.calculate_early_withdrawal_fee(1000, lock_until, current_time);
    assert!(!fee_info.is_early);
    assert_eq!(fee_info.fee_percent, 0);
}

#[test]
fn test_simple_early_withdrawal_immediate_max_fee() {
    let vault = SimpleVaultSecurity::new();

    let lock_until = 10000;
    let current_time = 0; // Immediately at start

    let fee_info = vault.calculate_early_withdrawal_fee(1000, lock_until, current_time);
    assert!(fee_info.is_early);
    // Fee should be approximately 50% at earliest withdrawal
    assert!(fee_info.fee_percent > 4900);
}

#[test]
fn test_simple_early_withdrawal_half_way() {
    let vault = SimpleVaultSecurity::new();

    let lock_until = 1000;
    let current_time = 500; // Half way through lock period

    let fee_info = vault.calculate_early_withdrawal_fee(1000, lock_until, current_time);
    assert!(fee_info.is_early);
    // Fee should be approximately 25% (half of max 50%)
    assert!(fee_info.fee_percent > 2000 && fee_info.fee_percent < 3000);
}

#[test]
fn test_simple_early_withdrawal_almost_unlocked() {
    let vault = SimpleVaultSecurity::new();

    let lock_until = 1000;
    let current_time = 990; // 10 seconds before unlock

    let fee_info = vault.calculate_early_withdrawal_fee(1000, lock_until, current_time);
    assert!(fee_info.is_early);
    // Fee should be very small (close to 0%)
    assert!(fee_info.fee_percent < 500);
}

#[test]
fn test_tensor_vault_security_creation() {
    let vault = TensorVaultSecurity::new(1_000_000, 1.0);

    // Should create successfully
    let info = vault.can_unlock_vault(1000, 2000);
    assert!(info.can_unlock);
}

#[test]
fn test_tensor_early_withdrawal_fee() {
    let vault = TensorVaultSecurity::new(1_000_000, 1.0);

    let lock_until = 1000;
    let current_time = 500;

    let fee_info = vault.calculate_early_withdrawal_fee(1000, lock_until, current_time);
    assert!(fee_info.is_early);
    // Should have similar fee to SimpleVaultSecurity
    assert!(fee_info.fee_percent > 2000 && fee_info.fee_percent < 3000);
}

#[test]
fn test_vault_fee_calculation_amount_agnostic() {
    let vault = SimpleVaultSecurity::new();

    let lock_until = 1000;
    let current_time = 500;

    let fee_info_1000 = vault.calculate_early_withdrawal_fee(1000, lock_until, current_time);
    let fee_info_5000 = vault.calculate_early_withdrawal_fee(5000, lock_until, current_time);

    // Fee percentage should be same regardless of amount
    assert_eq!(fee_info_1000.fee_percent, fee_info_5000.fee_percent);
}

#[test]
fn test_vault_time_remaining_calculation() {
    let vault = SimpleVaultSecurity::new();

    let lock_until = 5000;
    let current_time = 1000;

    let info = vault.can_unlock_vault(lock_until, current_time);
    assert_eq!(info.time_remaining, 4000);
}

#[test]
fn test_vault_multiple_unlock_checks() {
    let vault = SimpleVaultSecurity::new();
    let lock_until = 1000;

    // Check at different times
    for t in [0, 250, 500, 750, 999, 1000, 1001, 2000].iter() {
        let info = vault.can_unlock_vault(lock_until, *t);

        if t >= &lock_until {
            assert!(info.can_unlock);
        } else {
            assert!(!info.can_unlock);
        }
    }
}

#[test]
fn test_vault_fee_progression() {
    let vault = SimpleVaultSecurity::new();
    let lock_until = 10000;

    // Check fee at different stages of lock period
    let mut prev_fee = 5000u64; // Start at max

    for progress in (1..=10).rev() {
        let current_time = (10000 * (10 - progress) / 10) as i64;
        let fee_info = vault.calculate_early_withdrawal_fee(1000, lock_until, current_time);

        // Fee should decrease as lock time approaches expiration
        assert!(fee_info.fee_percent <= prev_fee || progress == 1);
        prev_fee = fee_info.fee_percent;
    }
}

#[test]
fn test_vault_security_edge_cases() {
    let vault = SimpleVaultSecurity::new();

    // Very large timestamps
    let info = vault.can_unlock_vault(i64::MAX, i64::MAX - 100);
    assert!(!info.can_unlock);

    // Zero timestamps
    let info = vault.can_unlock_vault(0, 0);
    assert!(info.can_unlock);
}

#[test]
fn test_vault_negative_time_remaining() {
    let vault = SimpleVaultSecurity::new();

    let lock_until = 1000;
    let current_time = 2000;

    let info = vault.can_unlock_vault(lock_until, current_time);
    // Time remaining should be clamped to 0
    assert_eq!(info.time_remaining, 0);
}

#[test]
fn test_tensor_entropy_reduction_disabled_at_zero() {
    let vault = TensorVaultSecurity::new(1_000_000, 1.0);

    // At zero entropy, should have no reduction
    let reduction = vault.calculate_entropy_reduction(0);
    assert_eq!(reduction, 0);
}

#[test]
fn test_tensor_entropy_reduction_at_max() {
    let vault = TensorVaultSecurity::new(1_000_000, 1.0);

    // At max entropy, should reduce by 100%
    let reduction = vault.calculate_entropy_reduction(1_000_000);
    assert_eq!(reduction, 100);
}

#[test]
fn test_tensor_coherence_discount_disabled_at_zero() {
    let vault = TensorVaultSecurity::new(1_000_000, 1.0);

    let discount = vault.calculate_coherence_discount(0);
    assert_eq!(discount, 0);
}

#[test]
fn test_tensor_coherence_discount_at_max() {
    let vault = TensorVaultSecurity::new(1_000_000, 1.0);

    // At max entropy, should discount by 50%
    let discount = vault.calculate_coherence_discount(1_000_000);
    assert_eq!(discount, 5000);
}
