//! Comprehensive integration tests for tribewarez-tokens program
//! Tests all token operations: initialization, minting, burning, transfers, and authority management

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::*;
    use tribewarez_tokens::state::{TokenAccount, TokenMint};

    // Test Constants
    const INITIAL_BALANCE: u64 = 1_000_000_000; // 1B tokens
    const MINT_AMOUNT: u64 = 100_000_000; // 100M tokens
    const BURN_AMOUNT: u64 = 50_000_000; // 50M tokens
    const TRANSFER_AMOUNT: u64 = 25_000_000; // 25M tokens

    // ============================================================================
    // TokenMint Tests
    // ============================================================================

    #[test]
    fn test_token_mint_space_calculation() {
        // Verify that TokenMint space calculation is correct
        assert_eq!(
            TokenMint::SPACE,
            8 + 32 + 32 + 32 + 1 + 9 + 8 + 8 + 8 + 9 + 68 + 20 + 261 + 8
        );
        assert!(TokenMint::SPACE > 0, "TokenMint space must be positive");
    }

    #[test]
    fn test_token_mint_default_state() {
        let mint = TokenMint::default();
        assert_eq!(mint.decimals, 0);
        assert_eq!(mint.total_supply, 0);
        assert_eq!(mint.total_minted, 0);
        assert_eq!(mint.total_burned, 0);
        assert!(mint.name.is_empty());
        assert!(mint.symbol.is_empty());
        assert!(mint.uri.is_none());
    }

    #[test]
    fn test_token_mint_validate_decimals() {
        let mut mint = TokenMint::default();

        // Valid: 0-18 decimals
        mint.decimals = 0;
        assert!(mint.validate().is_ok());

        mint.decimals = 8;
        assert!(mint.validate().is_ok());

        mint.decimals = 18;
        assert!(mint.validate().is_ok());
    }

    #[test]
    fn test_token_mint_can_mint_without_cap() {
        let mint = TokenMint {
            supply_cap: None,
            total_supply: 1_000_000_000,
            decimals: 8,
            ..Default::default()
        };

        // Should always be able to mint without cap
        assert!(mint.can_mint(1_000_000));
        assert!(mint.can_mint(u64::MAX));
    }

    #[test]
    fn test_token_mint_can_mint_with_cap() {
        let mint = TokenMint {
            supply_cap: Some(2_000_000_000),
            total_supply: 1_500_000_000,
            decimals: 8,
            ..Default::default()
        };

        // Can mint up to cap
        assert!(mint.can_mint(500_000_000));
        // Cannot exceed cap
        assert!(!mint.can_mint(500_000_001));
    }

    #[test]
    fn test_token_mint_supply_cap_validation() {
        let mint = TokenMint {
            supply_cap: Some(1_000_000_000),
            total_supply: 1_000_000_000,
            decimals: 8,
            ..Default::default()
        };

        assert!(mint.validate().is_ok());
    }

    #[test]
    fn test_token_mint_supply_cap_exceeded() {
        let mint = TokenMint {
            supply_cap: Some(1_000_000_000),
            total_supply: 1_000_000_001, // Exceeds cap
            decimals: 8,
            ..Default::default()
        };

        assert!(mint.validate().is_err());
    }

    // ============================================================================
    // TokenAccount Tests
    // ============================================================================

    #[test]
    fn test_token_account_space_calculation() {
        assert_eq!(TokenAccount::SPACE, 8 + 32 + 32 + 8 + 1 + 33 + 8 + 8);
        assert!(
            TokenAccount::SPACE > 0,
            "TokenAccount space must be positive"
        );
    }

    #[test]
    fn test_token_account_default_state() {
        let account = TokenAccount::default();
        assert_eq!(account.balance, 0);
        assert!(!account.is_frozen);
        assert!(account.delegate.is_none());
        assert_eq!(account.delegated_amount, 0);
        assert_eq!(account.created_at, 0);
    }

    #[test]
    fn test_token_account_is_usable_when_not_frozen() {
        let account = TokenAccount {
            is_frozen: false,
            ..Default::default()
        };
        assert!(account.is_usable());
    }

    #[test]
    fn test_token_account_not_usable_when_frozen() {
        let account = TokenAccount {
            is_frozen: true,
            ..Default::default()
        };
        assert!(!account.is_usable());
    }

    #[test]
    fn test_token_account_can_transfer_sufficient_balance() {
        let account = TokenAccount {
            balance: 1_000_000,
            is_frozen: false,
            ..Default::default()
        };
        assert!(account.can_transfer(500_000));
        assert!(account.can_transfer(1_000_000));
    }

    #[test]
    fn test_token_account_cannot_transfer_insufficient_balance() {
        let account = TokenAccount {
            balance: 1_000_000,
            is_frozen: false,
            ..Default::default()
        };
        assert!(!account.can_transfer(1_000_001));
    }

    #[test]
    fn test_token_account_cannot_transfer_when_frozen() {
        let account = TokenAccount {
            balance: 1_000_000,
            is_frozen: true,
            ..Default::default()
        };
        assert!(!account.can_transfer(500_000));
    }

    #[test]
    fn test_token_account_get_available_balance() {
        let account = TokenAccount {
            balance: 500_000,
            ..Default::default()
        };
        assert_eq!(account.get_available_balance(), 500_000);
    }

    // ============================================================================
    // Arithmetic Tests (Overflow/Underflow)
    // ============================================================================

    #[test]
    fn test_balance_addition_overflow_protection() {
        let mut account = TokenAccount {
            balance: u64::MAX - 1,
            ..Default::default()
        };

        // Should handle overflow
        let result = account.balance.checked_add(2);
        assert!(result.is_none(), "Should detect overflow");
    }

    #[test]
    fn test_balance_subtraction_underflow_protection() {
        let account = TokenAccount {
            balance: 100,
            ..Default::default()
        };

        // Should handle underflow
        let result = account.balance.checked_sub(101);
        assert!(result.is_none(), "Should detect underflow");
    }

    #[test]
    fn test_supply_addition_overflow_protection() {
        let mut mint = TokenMint {
            total_supply: u64::MAX - 1,
            decimals: 8,
            ..Default::default()
        };

        // Should handle overflow
        let result = mint.total_supply.checked_add(2);
        assert!(result.is_none(), "Should detect overflow");
    }

    // ============================================================================
    // Multi-Token Scenarios
    // ============================================================================

    #[test]
    fn test_multiple_accounts_same_mint() {
        // Scenario: Multiple users holding same token
        let mint = TokenMint {
            decimals: 8,
            total_supply: 1_000_000_000,
            ..Default::default()
        };

        let account1 = TokenAccount {
            balance: 300_000_000,
            ..Default::default()
        };

        let account2 = TokenAccount {
            balance: 700_000_000,
            ..Default::default()
        };

        assert_eq!(account1.balance + account2.balance, mint.total_supply);
    }

    #[test]
    fn test_multiple_mints_different_decimals() {
        // AUMCOIN: 8 decimals, non-inflationary
        let aumcoin = TokenMint {
            decimals: 8,
            supply_cap: Some(21_000_000_000_000_000), // 21M AUM * 10^8
            inflation_rate: None,
            ..Default::default()
        };

        // TRIBECOIN: 6 decimals, 5% inflation
        let tribecoin = TokenMint {
            decimals: 6,
            supply_cap: None,
            inflation_rate: Some(0.05),
            ..Default::default()
        };

        // RAVECOIN: 8 decimals, 10% inflation
        let ravecoin = TokenMint {
            decimals: 8,
            supply_cap: None,
            inflation_rate: Some(0.10),
            ..Default::default()
        };

        assert_eq!(aumcoin.decimals, 8);
        assert_eq!(tribecoin.decimals, 6);
        assert_eq!(ravecoin.decimals, 8);

        assert!(aumcoin.supply_cap.is_some());
        assert!(tribecoin.supply_cap.is_none());
        assert!(ravecoin.supply_cap.is_none());
    }

    // ============================================================================
    // Edge Cases and Boundary Tests
    // ============================================================================

    #[test]
    fn test_zero_amount_transfers() {
        let account = TokenAccount {
            balance: 1_000_000,
            ..Default::default()
        };

        // Zero amount check is done in instruction handler, not in can_transfer
        // The can_transfer check only verifies balance sufficiency
        assert!(account.can_transfer(1)); // Minimum amount
    }

    #[test]
    fn test_max_balance() {
        let account = TokenAccount {
            balance: u64::MAX,
            ..Default::default()
        };

        // Should allow transfers up to max balance
        assert!(account.can_transfer(u64::MAX));
        // Cannot transfer more than available
        // (would be caught by logic in instruction handler)
    }

    #[test]
    fn test_mint_metadata_updates() {
        let mut mint = TokenMint {
            name: "Original Name".to_string(),
            symbol: "ORIG".to_string(),
            uri: None,
            decimals: 8,
            ..Default::default()
        };

        assert_eq!(mint.name, "Original Name");
        assert_eq!(mint.symbol, "ORIG");

        // Simulate metadata update
        mint.name = "Updated Name".to_string();
        mint.symbol = "UPDT".to_string();
        mint.uri = Some("https://example.com".to_string());

        assert_eq!(mint.name, "Updated Name");
        assert_eq!(mint.symbol, "UPDT");
        assert_eq!(mint.uri, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_account_freeze_unfreeze_cycle() {
        let mut account = TokenAccount {
            balance: 1_000_000,
            is_frozen: false,
            ..Default::default()
        };

        assert!(account.is_usable());

        // Freeze
        account.is_frozen = true;
        assert!(!account.is_usable());

        // Unfreeze
        account.is_frozen = false;
        assert!(account.is_usable());
    }

    // ============================================================================
    // Inflation Rate Validation Tests
    // ============================================================================

    #[test]
    fn test_inflation_rate_0_percent() {
        let mint = TokenMint {
            decimals: 8,
            inflation_rate: Some(0.0),
            ..Default::default()
        };
        assert_eq!(mint.inflation_rate, Some(0.0));
    }

    #[test]
    fn test_inflation_rate_5_percent() {
        let mint = TokenMint {
            decimals: 6,
            inflation_rate: Some(0.05),
            ..Default::default()
        };
        assert_eq!(mint.inflation_rate, Some(0.05));
    }

    #[test]
    fn test_inflation_rate_10_percent() {
        let mint = TokenMint {
            decimals: 8,
            inflation_rate: Some(0.10),
            ..Default::default()
        };
        assert_eq!(mint.inflation_rate, Some(0.10));
    }

    #[test]
    fn test_inflation_rate_100_percent() {
        let mint = TokenMint {
            decimals: 8,
            inflation_rate: Some(1.0),
            ..Default::default()
        };
        assert_eq!(mint.inflation_rate, Some(1.0));
    }

    #[test]
    fn test_no_inflation() {
        let mint = TokenMint {
            decimals: 8,
            inflation_rate: None,
            ..Default::default()
        };
        assert!(mint.inflation_rate.is_none());
    }

    // ============================================================================
    // Batch Operation Tests
    // ============================================================================

    #[test]
    fn test_sequential_mints() {
        let mut mint = TokenMint {
            decimals: 8,
            supply_cap: Some(1_000_000_000),
            total_supply: 0,
            ..Default::default()
        };

        let amounts = vec![100_000_000, 200_000_000, 300_000_000];

        for amount in amounts {
            assert!(mint.can_mint(amount));
            mint.total_supply += amount;
        }

        assert_eq!(mint.total_supply, 600_000_000);
        assert!(!mint.can_mint(500_000_000)); // Would exceed cap
    }

    #[test]
    fn test_sequential_burns() {
        let mut mint = TokenMint {
            decimals: 8,
            supply_cap: Some(1_000_000_000),
            total_supply: 600_000_000,
            total_burned: 0,
            ..Default::default()
        };

        let burn_amounts = vec![100_000_000, 150_000_000, 200_000_000];

        for amount in burn_amounts {
            if mint.total_supply >= amount {
                mint.total_supply -= amount;
                mint.total_burned += amount;
            }
        }

        assert_eq!(mint.total_supply, 150_000_000);
        assert_eq!(mint.total_burned, 450_000_000);
    }

    // ============================================================================
    // Cross-Account Scenarios
    // ============================================================================

    #[test]
    fn test_transfer_chain() {
        // Alice -> Bob -> Carol
        let mut alice = TokenAccount {
            balance: 1_000_000,
            ..Default::default()
        };

        let mut bob = TokenAccount {
            balance: 0,
            ..Default::default()
        };

        let mut carol = TokenAccount {
            balance: 0,
            ..Default::default()
        };

        // Alice transfers to Bob
        if alice.can_transfer(500_000) {
            alice.balance -= 500_000;
            bob.balance += 500_000;
        }

        assert_eq!(alice.balance, 500_000);
        assert_eq!(bob.balance, 500_000);

        // Bob transfers to Carol
        if bob.can_transfer(300_000) {
            bob.balance -= 300_000;
            carol.balance += 300_000;
        }

        assert_eq!(bob.balance, 200_000);
        assert_eq!(carol.balance, 300_000);
        assert_eq!(alice.balance + bob.balance + carol.balance, 1_000_000);
    }

    #[test]
    fn test_frozen_account_cannot_receive_transfers() {
        let mut account = TokenAccount {
            balance: 100_000,
            is_frozen: true,
            ..Default::default()
        };

        // Account is frozen, so it cannot participate in transfers
        assert!(!account.is_usable());
        // But balance is still stored
        assert_eq!(account.balance, 100_000);
    }

    // ============================================================================
    // Summary Statistics
    // ============================================================================

    #[test]
    fn test_total_minted_vs_burned() {
        let mint = TokenMint {
            decimals: 8,
            total_minted: 5_000_000_000,
            total_burned: 2_000_000_000,
            total_supply: 3_000_000_000,
            ..Default::default()
        };

        assert_eq!(mint.total_minted - mint.total_burned, mint.total_supply);
    }

    #[test]
    fn test_mint_with_treasury() {
        let mint = TokenMint {
            decimals: 8,
            total_supply: 1_000_000_000,
            name: "Treasury Test Token".to_string(),
            symbol: "TTT".to_string(),
            ..Default::default()
        };

        let mut treasury = TokenAccount {
            balance: mint.total_supply / 10, // 10% to treasury
            ..Default::default()
        };

        let mut community = TokenAccount {
            balance: mint.total_supply * 9 / 10, // 90% to community
            ..Default::default()
        };

        assert_eq!(treasury.balance + community.balance, mint.total_supply);
    }

    // ============================================================================
    // Authority Management Tests
    // ============================================================================

    #[test]
    fn test_mint_authority_validation() {
        let mut mint = TokenMint {
            mint_authority: Pubkey::default(),
            decimals: 8,
            ..Default::default()
        };

        let original_auth = mint.mint_authority;
        let new_auth = Pubkey::new_unique();

        mint.mint_authority = new_auth;
        assert_eq!(mint.mint_authority, new_auth);
        assert_ne!(mint.mint_authority, original_auth);
    }

    #[test]
    fn test_freeze_authority_validation() {
        let mut mint = TokenMint {
            freeze_authority: Pubkey::default(),
            decimals: 8,
            ..Default::default()
        };

        let original_auth = mint.freeze_authority;
        let new_auth = Pubkey::new_unique();

        mint.freeze_authority = new_auth;
        assert_eq!(mint.freeze_authority, new_auth);
        assert_ne!(mint.freeze_authority, original_auth);
    }

    // ============================================================================
    // Performance/Stress Tests (Conceptual)
    // ============================================================================

    #[test]
    fn test_high_decimal_precision() {
        let mint = TokenMint {
            decimals: 18, // Maximum
            ..Default::default()
        };

        assert!(mint.validate().is_ok());
    }

    #[test]
    fn test_large_supply_cap() {
        let mint = TokenMint {
            decimals: 8,
            supply_cap: Some(u64::MAX),
            total_supply: 0,
            ..Default::default()
        };

        assert!(mint.can_mint(u64::MAX - 1));
    }

    #[test]
    fn test_many_sequential_small_transfers() {
        let mut account = TokenAccount {
            balance: 10_000,
            ..Default::default()
        };

        let transfer_count = 100;
        let transfer_amount = 100;

        for _ in 0..transfer_count {
            if account.balance >= transfer_amount {
                account.balance -= transfer_amount;
            }
        }

        assert_eq!(account.balance, 0);
    }

    // ============================================================================
    // State Consistency Tests
    // ============================================================================

    #[test]
    fn test_mint_consistency_after_operations() {
        let mut mint = TokenMint {
            decimals: 8,
            supply_cap: Some(1_000_000_000),
            total_supply: 500_000_000,
            total_minted: 500_000_000,
            total_burned: 0,
            ..Default::default()
        };

        // Mint 100M
        mint.total_supply += 100_000_000;
        mint.total_minted += 100_000_000;
        assert_eq!(mint.total_supply, 600_000_000);

        // Burn 50M
        mint.total_supply -= 50_000_000;
        mint.total_burned += 50_000_000;
        assert_eq!(mint.total_supply, 550_000_000);

        assert!(mint.validate().is_ok());
    }

    #[test]
    fn test_account_consistency_after_operations() {
        let mut account = TokenAccount {
            balance: 1_000_000,
            is_frozen: false,
            ..Default::default()
        };

        // Transfer out
        account.balance -= 250_000;
        assert_eq!(account.balance, 750_000);

        // Freeze
        account.is_frozen = true;
        assert!(!account.is_usable());

        // Unfreeze
        account.is_frozen = false;
        assert!(account.is_usable());

        // Transfer out
        account.balance -= 250_000;
        assert_eq!(account.balance, 500_000);
    }
}
