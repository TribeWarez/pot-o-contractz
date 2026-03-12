#[cfg(test)]
mod tests {
    use anchor_lang::prelude::*;
    use tribewarez_bridge::state::{BridgeVault, WrappedToken};

    fn create_vault() -> BridgeVault {
        BridgeVault {
            vault_authority: Pubkey::new_unique(),
            token_a: Pubkey::new_unique(),
            token_b: Pubkey::new_unique(),
            fee_bps: 100, // 1%
            is_paused: false,
            collateral_balance: 0,
            wrapped_supply: 0,
            collected_fees: 0,
            bump: 0,
        }
    }

    // === Vault Initialization Tests ===

    #[test]
    fn test_vault_creation_and_initialization() {
        let vault = create_vault();
        assert_eq!(vault.collateral_balance, 0);
        assert_eq!(vault.wrapped_supply, 0);
        assert_eq!(vault.collected_fees, 0);
        assert!(!vault.is_paused);
        assert_eq!(vault.fee_bps, 100);
    }

    #[test]
    fn test_vault_different_fee_rates() {
        let mut vault = create_vault();
        vault.fee_bps = 50; // 0.5%
        assert_eq!(vault.fee_bps, 50);

        vault.fee_bps = 500; // 5%
        assert_eq!(vault.fee_bps, 500);

        vault.fee_bps = 10000; // 100% (max)
        assert_eq!(vault.fee_bps, 10000);
    }

    #[test]
    fn test_vault_state_consistency() {
        let vault = create_vault();
        assert!(vault.is_operational());
        assert!(!vault.is_paused);
    }

    // === Fee Calculation Tests ===

    #[test]
    fn test_calculate_fee_basic() {
        let vault = create_vault(); // 1% fee
        let (fee, after) = vault.calculate_fee(1000).unwrap();
        assert_eq!(fee, 10);
        assert_eq!(after, 990);
    }

    #[test]
    fn test_calculate_fee_zero_amount() {
        let vault = create_vault();
        let (fee, after) = vault.calculate_fee(0).unwrap();
        assert_eq!(fee, 0);
        assert_eq!(after, 0);
    }

    #[test]
    fn test_calculate_fee_large_amount() {
        let vault = create_vault(); // 1% fee
        let amount = 1_000_000_000u64;
        let (fee, after) = vault.calculate_fee(amount).unwrap();
        assert_eq!(fee, 10_000_000);
        assert_eq!(after, 990_000_000);
    }

    #[test]
    fn test_calculate_fee_no_fee() {
        let mut vault = create_vault();
        vault.fee_bps = 0;
        let (fee, after) = vault.calculate_fee(1000).unwrap();
        assert_eq!(fee, 0);
        assert_eq!(after, 1000);
    }

    #[test]
    fn test_calculate_fee_high_fee() {
        let mut vault = create_vault();
        vault.fee_bps = 5000; // 50%
        let (fee, after) = vault.calculate_fee(1000).unwrap();
        assert_eq!(fee, 500);
        assert_eq!(after, 500);
    }

    // === Collateral Management Tests ===

    #[test]
    fn test_deposit_collateral() {
        let mut vault = create_vault();
        assert!(vault.deposit(1000).is_ok());
        assert_eq!(vault.collateral_balance, 1000);
    }

    #[test]
    fn test_deposit_multiple_times() {
        let mut vault = create_vault();
        assert!(vault.deposit(1000).is_ok());
        assert!(vault.deposit(500).is_ok());
        assert!(vault.deposit(250).is_ok());
        assert_eq!(vault.collateral_balance, 1750);
    }

    #[test]
    fn test_withdraw_collateral() {
        let mut vault = create_vault();
        vault.collateral_balance = 1000;
        vault.wrapped_supply = 500;
        assert!(vault.withdraw(500).is_ok());
        assert_eq!(vault.collateral_balance, 500);
    }

    #[test]
    fn test_withdraw_insufficient_collateral() {
        let mut vault = create_vault();
        vault.collateral_balance = 100;
        assert!(vault.withdraw(200).is_err());
    }

    #[test]
    fn test_withdraw_maintains_backing() {
        let mut vault = create_vault();
        vault.collateral_balance = 1000;
        vault.wrapped_supply = 900;
        // Should succeed - can withdraw 100
        assert!(vault.withdraw(100).is_ok());
        assert_eq!(vault.collateral_balance, 900);

        // Should fail - would drop below backing ratio
        assert!(vault.withdraw(100).is_err());
    }

    #[test]
    fn test_validate_collateral_sufficient() {
        let mut vault = create_vault();
        vault.collateral_balance = 1000;
        vault.wrapped_supply = 500;
        assert!(vault.validate_collateral().is_ok());
    }

    #[test]
    fn test_validate_collateral_exact() {
        let mut vault = create_vault();
        vault.collateral_balance = 1000;
        vault.wrapped_supply = 1000;
        assert!(vault.validate_collateral().is_ok());
    }

    #[test]
    fn test_validate_collateral_insufficient() {
        let mut vault = create_vault();
        vault.collateral_balance = 500;
        vault.wrapped_supply = 1000;
        assert!(vault.validate_collateral().is_err());
    }

    #[test]
    fn test_collateral_overflow_protection() {
        let mut vault = create_vault();
        vault.collateral_balance = u64::MAX;
        // Should fail on overflow
        assert!(vault.deposit(1).is_err());
    }

    // === Wrapped Token Operations ===

    #[test]
    fn test_mint_wrapped_tokens() {
        let mut vault = create_vault();
        assert!(vault.mint_wrapped(1000).is_ok());
        assert_eq!(vault.wrapped_supply, 1000);
    }

    #[test]
    fn test_mint_multiple_wrapped_tokens() {
        let mut vault = create_vault();
        assert!(vault.mint_wrapped(500).is_ok());
        assert!(vault.mint_wrapped(300).is_ok());
        assert_eq!(vault.wrapped_supply, 800);
    }

    #[test]
    fn test_burn_wrapped_tokens() {
        let mut vault = create_vault();
        vault.wrapped_supply = 1000;
        assert!(vault.burn_wrapped(300).is_ok());
        assert_eq!(vault.wrapped_supply, 700);
    }

    #[test]
    fn test_burn_insufficient_wrapped() {
        let mut vault = create_vault();
        vault.wrapped_supply = 100;
        assert!(vault.burn_wrapped(200).is_err());
    }

    #[test]
    fn test_mint_wrapped_overflow() {
        let mut vault = create_vault();
        vault.wrapped_supply = u64::MAX;
        assert!(vault.mint_wrapped(1).is_err());
    }

    #[test]
    fn test_wrapped_supply_cycles() {
        let mut vault = create_vault();
        assert!(vault.mint_wrapped(1000).is_ok());
        assert_eq!(vault.wrapped_supply, 1000);
        assert!(vault.burn_wrapped(500).is_ok());
        assert_eq!(vault.wrapped_supply, 500);
        assert!(vault.mint_wrapped(250).is_ok());
        assert_eq!(vault.wrapped_supply, 750);
    }

    // === Fee Collection Tests ===

    #[test]
    fn test_collect_fees() {
        let mut vault = create_vault();
        assert!(vault.collect_fee(50).is_ok());
        assert_eq!(vault.collected_fees, 50);
    }

    #[test]
    fn test_collect_multiple_fees() {
        let mut vault = create_vault();
        assert!(vault.collect_fee(10).is_ok());
        assert!(vault.collect_fee(20).is_ok());
        assert!(vault.collect_fee(15).is_ok());
        assert_eq!(vault.collected_fees, 45);
    }

    #[test]
    fn test_withdraw_fees() {
        let mut vault = create_vault();
        vault.collected_fees = 100;
        assert!(vault.withdraw_fees(30).is_ok());
        assert_eq!(vault.collected_fees, 70);
    }

    #[test]
    fn test_withdraw_insufficient_fees() {
        let mut vault = create_vault();
        vault.collected_fees = 50;
        assert!(vault.withdraw_fees(100).is_err());
    }

    #[test]
    fn test_fee_accumulation_and_withdrawal() {
        let mut vault = create_vault();
        assert!(vault.collect_fee(100).is_ok());
        assert!(vault.collect_fee(50).is_ok());
        assert_eq!(vault.collected_fees, 150);
        assert!(vault.withdraw_fees(75).is_ok());
        assert_eq!(vault.collected_fees, 75);
    }

    #[test]
    fn test_fee_overflow_protection() {
        let mut vault = create_vault();
        vault.collected_fees = u64::MAX;
        assert!(vault.collect_fee(1).is_err());
    }

    // === Pause/Resume Operations ===

    #[test]
    fn test_pause_vault() {
        let mut vault = create_vault();
        assert!(!vault.is_paused);
        vault.pause();
        assert!(vault.is_paused);
        assert!(!vault.is_operational());
    }

    #[test]
    fn test_resume_vault() {
        let mut vault = create_vault();
        vault.pause();
        assert!(vault.is_paused);
        vault.resume();
        assert!(!vault.is_paused);
        assert!(vault.is_operational());
    }

    #[test]
    fn test_toggle_pause_multiple() {
        let mut vault = create_vault();
        for i in 0..5 {
            let expected = i % 2 == 0;
            if i % 2 == 0 {
                vault.pause();
            } else {
                vault.resume();
            }
            assert_eq!(vault.is_paused, expected);
        }
    }

    // === Complex Scenarios ===

    #[test]
    fn test_deposit_withdraw_cycle() {
        let mut vault = create_vault();
        assert!(vault.deposit(1000).is_ok());
        assert_eq!(vault.collateral_balance, 1000);
        assert!(vault.withdraw(500).is_ok());
        assert_eq!(vault.collateral_balance, 500);
    }

    #[test]
    fn test_vault_with_fees_deposit() {
        let mut vault = create_vault(); // 1% fee
        vault.collateral_balance = 0;
        vault.wrapped_supply = 0;

        let (fee, deposit_amount) = vault.calculate_fee(1000).unwrap();
        assert!(vault.deposit(deposit_amount).is_ok());
        assert!(vault.collect_fee(fee).is_ok());
        assert_eq!(vault.collateral_balance, 990);
        assert_eq!(vault.collected_fees, 10);
    }

    #[test]
    fn test_vault_with_fees_withdraw() {
        let mut vault = create_vault();
        vault.collateral_balance = 1000;
        vault.wrapped_supply = 500;

        let (fee, withdraw_amount) = vault.calculate_fee(500).unwrap();
        assert!(vault.burn_wrapped(withdraw_amount).is_ok());
        assert!(vault.withdraw(withdraw_amount).is_ok());
        assert!(vault.collect_fee(fee).is_ok());
    }

    #[test]
    fn test_multiple_vaults_independence() {
        let mut vault1 = create_vault();
        let mut vault2 = create_vault();

        assert!(vault1.deposit(1000).is_ok());
        assert!(vault2.deposit(2000).is_ok());

        assert_eq!(vault1.collateral_balance, 1000);
        assert_eq!(vault2.collateral_balance, 2000);

        assert!(vault1.withdraw(500).is_ok());
        assert_eq!(vault1.collateral_balance, 500);
        assert_eq!(vault2.collateral_balance, 2000);
    }

    #[test]
    fn test_vault_full_lifecycle() {
        let mut vault = create_vault();

        // Initialize deposits
        assert!(vault.deposit(5000).is_ok());
        assert!(vault.mint_wrapped(4500).is_ok());

        // Collect some fees
        assert!(vault.collect_fee(100).is_ok());
        assert!(vault.collect_fee(50).is_ok());

        // Pause operations
        vault.pause();
        assert!(!vault.is_operational());

        // Resume and continue
        vault.resume();
        assert!(vault.is_operational());

        // Validate final state
        assert_eq!(vault.collateral_balance, 5000);
        assert_eq!(vault.wrapped_supply, 4500);
        assert_eq!(vault.collected_fees, 150);
        assert!(vault.validate_collateral().is_ok());
    }

    #[test]
    fn test_vault_stress_large_operations() {
        let mut vault = create_vault();
        let large_amount = 1_000_000_000_000u64; // 1 trillion

        assert!(vault.deposit(large_amount).is_ok());
        assert_eq!(vault.collateral_balance, large_amount);

        let (fee, _after) = vault.calculate_fee(large_amount).unwrap();
        assert!(vault.collect_fee(fee).is_ok());

        let withdraw_amount = large_amount / 2;
        assert!(vault.withdraw(withdraw_amount).is_ok());
    }

    #[test]
    fn test_vault_authority_tracking() {
        let authority = Pubkey::new_unique();
        let mut vault = create_vault();
        vault.vault_authority = authority;

        assert_eq!(vault.vault_authority, authority);
    }

    #[test]
    fn test_vault_token_pair_tracking() {
        let token_a = Pubkey::new_unique();
        let token_b = Pubkey::new_unique();
        let mut vault = create_vault();
        vault.token_a = token_a;
        vault.token_b = token_b;

        assert_eq!(vault.token_a, token_a);
        assert_eq!(vault.token_b, token_b);
    }

    #[test]
    fn test_sequential_deposits_and_withdrawals() {
        let mut vault = create_vault();

        // Sequence of operations
        let ops = vec![
            (1000, true), // deposit
            (500, false), // withdraw
            (750, true),  // deposit
            (200, false), // withdraw
        ];

        let mut balance = 0u64;
        for (amount, is_deposit) in ops {
            if is_deposit {
                assert!(vault.deposit(amount).is_ok());
                balance += amount;
            } else {
                assert!(vault.withdraw(amount).is_ok());
                balance -= amount;
            }
            assert_eq!(vault.collateral_balance, balance);
        }
    }

    #[test]
    fn test_vault_edge_case_zero_fee() {
        let mut vault = create_vault();
        vault.fee_bps = 0;
        let (fee, after) = vault.calculate_fee(5000).unwrap();
        assert_eq!(fee, 0);
        assert_eq!(after, 5000);
    }

    #[test]
    fn test_vault_edge_case_max_fee() {
        let mut vault = create_vault();
        vault.fee_bps = 10000; // 100%
        let (fee, after) = vault.calculate_fee(1000).unwrap();
        assert_eq!(fee, 1000);
        assert_eq!(after, 0);
    }

    #[test]
    fn test_vault_collateral_various_ratios() {
        for ratio in &[0.5, 0.75, 1.0, 1.5, 2.0] {
            let mut vault = create_vault();
            vault.wrapped_supply = 1000;
            vault.collateral_balance = (1000.0 * ratio) as u64;

            if *ratio < 1.0 {
                assert!(vault.validate_collateral().is_err());
            } else {
                assert!(vault.validate_collateral().is_ok());
            }
        }
    }

    #[test]
    fn test_wrapped_token_creation() {
        let token = WrappedToken {
            original_token: Pubkey::new_unique(),
            wrapped_mint: Pubkey::new_unique(),
            vault: Pubkey::new_unique(),
            total_wrapped: 0,
            decimals: 8,
            name: "Wrapped NMTC".to_string(),
            symbol: "wNMTC".to_string(),
            bump: 0,
        };

        assert_eq!(token.decimals, 8);
        assert_eq!(token.total_wrapped, 0);
    }

    #[test]
    fn test_wrapped_token_metadata() {
        let original = Pubkey::new_unique();
        let wrapped = Pubkey::new_unique();
        let vault = Pubkey::new_unique();

        let token = WrappedToken {
            original_token: original,
            wrapped_mint: wrapped,
            vault,
            total_wrapped: 5000,
            decimals: 6,
            name: "Wrapped PPTC".to_string(),
            symbol: "wPPTC".to_string(),
            bump: 0,
        };

        assert_eq!(token.original_token, original);
        assert_eq!(token.wrapped_mint, wrapped);
        assert_eq!(token.vault, vault);
    }

    #[test]
    fn test_vault_multi_token_operations() {
        let mut vault1 = create_vault();
        let mut vault2 = create_vault();

        vault1.token_a = Pubkey::new_unique();
        vault2.token_a = Pubkey::new_unique();

        assert!(vault1.deposit(1000).is_ok());
        assert!(vault2.deposit(2000).is_ok());
        assert!(vault1.mint_wrapped(800).is_ok());
        assert!(vault2.mint_wrapped(1800).is_ok());

        assert_eq!(vault1.collateral_balance, 1000);
        assert_eq!(vault2.collateral_balance, 2000);
        assert_eq!(vault1.wrapped_supply, 800);
        assert_eq!(vault2.wrapped_supply, 1800);

        assert!(vault1.validate_collateral().is_ok());
        assert!(vault2.validate_collateral().is_ok());
    }

    #[test]
    fn test_vault_fee_calculation_rounding() {
        let mut vault = create_vault();
        vault.fee_bps = 33; // 0.33%

        let (fee, after) = vault.calculate_fee(100).unwrap();
        assert_eq!(fee, 0);
        assert_eq!(after, 100);

        let (fee2, after2) = vault.calculate_fee(1000).unwrap();
        assert_eq!(fee2, 3);
        assert_eq!(after2, 997);
    }

    #[test]
    fn test_vault_sequential_fee_operations() {
        let mut vault = create_vault();
        vault.fee_bps = 100;

        vault.collect_fee(10).unwrap();
        vault.collect_fee(20).unwrap();
        vault.collect_fee(15).unwrap();

        assert_eq!(vault.collected_fees, 45);

        vault.withdraw_fees(20).unwrap();
        assert_eq!(vault.collected_fees, 25);

        vault.withdraw_fees(25).unwrap();
        assert_eq!(vault.collected_fees, 0);
    }

    #[test]
    fn test_vault_burn_exact_supply() {
        let mut vault = create_vault();
        vault.wrapped_supply = 1000;

        vault.burn_wrapped(1000).unwrap();
        assert_eq!(vault.wrapped_supply, 0);
    }

    #[test]
    fn test_vault_full_pause_resume_cycle() {
        let mut vault = create_vault();

        vault.pause();
        assert!(vault.is_paused);
        assert!(!vault.is_operational());

        vault.resume();
        assert!(!vault.is_paused);
        assert!(vault.is_operational());

        vault.pause();
        assert!(vault.is_paused);
        assert!(!vault.is_operational());

        vault.resume();
        assert!(!vault.is_paused);
        assert!(vault.is_operational());
    }

    #[test]
    fn test_vault_backing_ratio_checks() {
        let mut vault = create_vault();

        vault.collateral_balance = 1000;
        vault.wrapped_supply = 1000;
        assert!(vault.validate_collateral().is_ok());

        vault.wrapped_supply = 1001;
        assert!(vault.validate_collateral().is_err());

        vault.collateral_balance = 2000;
        vault.wrapped_supply = 1500;
        assert!(vault.validate_collateral().is_ok());
    }

    #[test]
    fn test_wrapped_token_length_constant() {
        let len = WrappedToken::LEN;
        assert!(len > 0);
    }

    #[test]
    fn test_vault_bump_tracking() {
        let mut vault = create_vault();
        vault.bump = 42;
        assert_eq!(vault.bump, 42);
    }

    #[test]
    fn test_vault_deposit_overflow_prevention() {
        let mut vault = create_vault();
        vault.collateral_balance = u64::MAX - 100;
        assert!(vault.deposit(200).is_err());
    }

    #[test]
    fn test_vault_wrapped_mint_overflow() {
        let mut vault = create_vault();
        vault.wrapped_supply = u64::MAX - 100;
        assert!(vault.mint_wrapped(200).is_err());
    }

    #[test]
    fn test_vault_deposit_zero() {
        let mut vault = create_vault();
        assert!(vault.deposit(0).is_ok());
        assert_eq!(vault.collateral_balance, 0);
    }

    #[test]
    fn test_vault_withdraw_zero() {
        let mut vault = create_vault();
        vault.collateral_balance = 1000;
        vault.wrapped_supply = 500;
        assert!(vault.withdraw(0).is_ok());
    }

    #[test]
    fn test_vault_token_pair_independence() {
        let token_a1 = Pubkey::new_unique();
        let token_b1 = Pubkey::new_unique();
        let token_a2 = Pubkey::new_unique();
        let token_b2 = Pubkey::new_unique();

        let mut vault1 = create_vault();
        vault1.token_a = token_a1;
        vault1.token_b = token_b1;

        let mut vault2 = create_vault();
        vault2.token_a = token_a2;
        vault2.token_b = token_b2;

        assert_ne!(vault1.token_a, vault2.token_a);
        assert_ne!(vault1.token_b, vault2.token_b);
    }

    #[test]
    fn test_vault_all_zero_balances() {
        let vault = create_vault();
        assert_eq!(vault.collateral_balance, 0);
        assert_eq!(vault.wrapped_supply, 0);
        assert_eq!(vault.collected_fees, 0);
    }
}
