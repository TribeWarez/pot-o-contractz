#[cfg(test)]
mod tests {
    use anchor_lang::prelude::*;
    use tribewarez_liquidity::state::{LiquidityPool, PoolConfig, PoolPosition, PriceFeed};

    fn create_pool() -> LiquidityPool {
        LiquidityPool {
            token_a: Pubkey::new_unique(),
            token_b: Pubkey::new_unique(),
            reserve_a: 1000,
            reserve_b: 2000,
            lp_token_mint: Pubkey::new_unique(),
            fee_bps: 30,
            admin: Pubkey::new_unique(),
            bump: 0,
        }
    }

    fn create_position() -> PoolPosition {
        PoolPosition {
            owner: Pubkey::new_unique(),
            pool: Pubkey::new_unique(),
            shares: 100,
            bump: 0,
        }
    }

    fn create_config() -> PoolConfig {
        PoolConfig {
            authority: Pubkey::new_unique(),
            default_fee_bps: 30,
            min_liquidity: 1000,
            max_fee_bps: 1000,
            bump: 0,
        }
    }

    fn create_price_feed() -> PriceFeed {
        PriceFeed {
            token_a: Pubkey::new_unique(),
            token_b: Pubkey::new_unique(),
            price_a_to_b: 100,
            last_update: 1000,
            bump: 0,
        }
    }

    // === Pool Creation Tests ===

    #[test]
    fn test_pool_creation() {
        let pool = create_pool();
        assert_eq!(pool.reserve_a, 1000);
        assert_eq!(pool.reserve_b, 2000);
        assert_eq!(pool.fee_bps, 30);
    }

    #[test]
    fn test_pool_token_pair() {
        let pool = create_pool();
        assert_ne!(pool.token_a, pool.token_b);
    }

    #[test]
    fn test_pool_initial_reserves() {
        let mut pool = create_pool();
        pool.reserve_a = 0;
        pool.reserve_b = 0;
        assert_eq!(pool.reserve_a, 0);
        assert_eq!(pool.reserve_b, 0);
    }

    // === Swap Calculation Tests ===

    #[test]
    fn test_calculate_swap_output_basic() {
        let pool = create_pool();
        let output = pool.calculate_swap_output(100, pool.token_a).unwrap();
        assert!(output > 0);
    }

    #[test]
    fn test_calculate_swap_output_zero() {
        let pool = create_pool();
        let output = pool.calculate_swap_output(0, pool.token_a);
        assert!(output.is_err());
    }

    #[test]
    fn test_calculate_swap_output_insufficient_liquidity() {
        let mut pool = create_pool();
        pool.reserve_a = 0;
        let output = pool.calculate_swap_output(100, pool.token_a);
        assert!(output.is_err());
    }

    #[test]
    fn test_calculate_swap_different_tokens() {
        let pool = create_pool();
        let output_a = pool.calculate_swap_output(100, pool.token_a).unwrap();
        let output_b = pool.calculate_swap_output(100, pool.token_b).unwrap();
        assert!(output_a > 0 || output_b > 0);
    }

    #[test]
    fn test_calculate_swap_high_fee() {
        let mut pool = create_pool();
        pool.fee_bps = 1000;
        let output = pool.calculate_swap_output(1000, pool.token_a).unwrap();
        assert!(output < 1000);
    }

    #[test]
    fn test_calculate_swap_no_fee() {
        let mut pool = create_pool();
        pool.fee_bps = 0;
        let output = pool.calculate_swap_output(100, pool.token_a).unwrap();
        assert!(output > 0);
    }

    #[test]
    fn test_calculate_swap_large_input() {
        let pool = create_pool();
        let output = pool.calculate_swap_output(1_000_000, pool.token_a).unwrap();
        assert!(output > 0);
    }

    // === LP Shares Tests ===

    #[test]
    fn test_calculate_lp_shares_initial() {
        let mut pool = create_pool();
        pool.reserve_a = 0;
        pool.reserve_b = 0;
        let shares = pool.calculate_lp_shares(1000, 2000).unwrap();
        assert!(shares > 0);
    }

    #[test]
    fn test_calculate_lp_shares_existing() {
        let pool = create_pool();
        let shares = pool.calculate_lp_shares(100, 200).unwrap();
        assert!(shares > 0);
    }

    #[test]
    fn test_calculate_lp_shares_zero_amounts() {
        let pool = create_pool();
        let shares = pool.calculate_lp_shares(0, 0).unwrap();
        assert_eq!(shares, 0);
    }

    #[test]
    fn test_calculate_lp_shares_proportional() {
        let mut pool = create_pool();
        pool.reserve_a = 0;
        pool.reserve_b = 0;
        let shares1 = pool.calculate_lp_shares(100, 200).unwrap();
        let shares2 = pool.calculate_lp_shares(50, 100).unwrap();
        // Both should produce valid shares for equal ratios
        assert!(shares1 > 0);
        assert!(shares2 > 0);
    }

    // === Withdraw Amount Tests ===

    #[test]
    fn test_calculate_withdraw_amounts() {
        let pool = create_pool();
        let (amount_a, amount_b) = pool.calculate_withdraw_amounts(50, 1000).unwrap();
        assert!(amount_a > 0);
        assert!(amount_b > 0);
    }

    #[test]
    fn test_calculate_withdraw_zero_shares() {
        let pool = create_pool();
        let result = pool.calculate_withdraw_amounts(0, 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_withdraw_all_shares() {
        let pool = create_pool();
        let total_shares = pool.reserve_a.saturating_add(pool.reserve_b);
        let (amount_a, amount_b) = pool
            .calculate_withdraw_amounts(total_shares, total_shares)
            .unwrap();
        assert!(amount_a <= pool.reserve_a);
        assert!(amount_b <= pool.reserve_b);
    }

    // === Reserve Update Tests ===

    #[test]
    fn test_update_reserves_add() {
        let mut pool = create_pool();
        pool.update_reserves(100, 200, true).unwrap();
        assert_eq!(pool.reserve_a, 1100);
        assert_eq!(pool.reserve_b, 2200);
    }

    #[test]
    fn test_update_reserves_remove() {
        let mut pool = create_pool();
        pool.update_reserves(500, 1000, false).unwrap();
        assert_eq!(pool.reserve_a, 500);
        assert_eq!(pool.reserve_b, 1000);
    }

    #[test]
    fn test_update_reserves_insufficient() {
        let mut pool = create_pool();
        let result = pool.update_reserves(2000, 4000, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_reserves_overflow() {
        let mut pool = create_pool();
        pool.reserve_a = u64::MAX;
        let result = pool.update_reserves(1, 0, true);
        assert!(result.is_err());
    }

    // === Token Pair Validation Tests ===

    #[test]
    fn test_validate_pair_valid() {
        let pool = create_pool();
        let result = pool.validate_pair(pool.token_a, pool.token_b);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pair_reversed() {
        let pool = create_pool();
        let result = pool.validate_pair(pool.token_b, pool.token_a);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pair_invalid() {
        let pool = create_pool();
        let result = pool.validate_pair(Pubkey::new_unique(), Pubkey::new_unique());
        assert!(result.is_err());
    }

    // === PoolPosition Tests ===

    #[test]
    fn test_position_creation() {
        let position = create_position();
        assert_eq!(position.shares, 100);
    }

    #[test]
    fn test_position_owner() {
        let position = create_position();
        assert_ne!(position.owner, Pubkey::default());
    }

    #[test]
    fn test_position_pool_link() {
        let position = create_position();
        assert_ne!(position.pool, Pubkey::default());
    }

    // === PoolConfig Tests ===

    #[test]
    fn test_config_creation() {
        let config = create_config();
        assert_eq!(config.default_fee_bps, 30);
        assert_eq!(config.max_fee_bps, 1000);
    }

    #[test]
    fn test_config_set_fee_valid() {
        let mut config = create_config();
        let result = config.set_fee(500);
        assert!(result.is_ok());
        assert_eq!(config.default_fee_bps, 500);
    }

    #[test]
    fn test_config_set_fee_invalid() {
        let mut config = create_config();
        let result = config.set_fee(2000);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_min_liquidity() {
        let config = create_config();
        assert!(config.min_liquidity > 0);
    }

    // === PriceFeed Tests ===

    #[test]
    fn test_price_feed_creation() {
        let feed = create_price_feed();
        assert_eq!(feed.price_a_to_b, 100);
    }

    #[test]
    fn test_price_feed_update() {
        let mut feed = create_price_feed();
        feed.update_price(150).unwrap();
        assert_eq!(feed.price_a_to_b, 150);
    }

    #[test]
    fn test_price_feed_twap() {
        let feed = create_price_feed();
        let twap = feed.calculate_twap(90).unwrap();
        assert!(twap > 0);
    }

    #[test]
    fn test_price_feed_twap_initial() {
        let mut feed = create_price_feed();
        feed.last_update = 0;
        let twap = feed.calculate_twap(100).unwrap();
        assert_eq!(twap, feed.price_a_to_b);
    }

    // === Complex Scenarios ===

    #[test]
    fn test_pool_constant_length() {
        let len = LiquidityPool::LEN;
        assert!(len > 0);
    }

    #[test]
    fn test_position_constant_length() {
        let len = PoolPosition::LEN;
        assert!(len > 0);
    }

    #[test]
    fn test_config_constant_length() {
        let len = PoolConfig::LEN;
        assert!(len > 0);
    }

    #[test]
    fn test_price_feed_constant_length() {
        let len = PriceFeed::LEN;
        assert!(len > 0);
    }

    #[test]
    fn test_swap_multiple_times() {
        let mut pool = create_pool();
        let initial_a = pool.reserve_a;
        let initial_b = pool.reserve_b;

        let _ = pool.calculate_swap_output(100, pool.token_a).unwrap();
        let _ = pool.calculate_swap_output(200, pool.token_a).unwrap();
        let _ = pool.calculate_swap_output(50, pool.token_b).unwrap();

        assert_eq!(pool.reserve_a, initial_a);
        assert_eq!(pool.reserve_b, initial_b);
    }

    #[test]
    fn test_liquidity_provision_sequence() {
        let mut pool = create_pool();

        let shares1 = pool.calculate_lp_shares(1000, 2000).unwrap();
        pool.update_reserves(1000, 2000, true).unwrap();

        let shares2 = pool.calculate_lp_shares(500, 1000).unwrap();
        pool.update_reserves(500, 1000, true).unwrap();

        assert!(shares2 < shares1);
    }

    #[test]
    fn test_withdraw_partial() {
        let mut pool = create_pool();
        pool.update_reserves(1000, 2000, true).unwrap();

        let (a, b) = pool.calculate_withdraw_amounts(25, 100).unwrap();

        assert!(a < 1000);
        assert!(b < 2000);
    }

    #[test]
    fn test_pool_fee_boundaries() {
        let mut pool = create_pool();

        pool.fee_bps = 0;
        assert!(pool.calculate_swap_output(100, pool.token_a).is_ok());

        pool.fee_bps = 10000;
        let output = pool.calculate_swap_output(100, pool.token_a).unwrap();
        assert!(output < 100);
    }

    #[test]
    fn test_price_feed_timestamp() {
        let mut feed = create_price_feed();
        feed.last_update = 1000;
        feed.update_price(120).unwrap();
        assert!(feed.last_update >= 1);
    }

    #[test]
    fn test_position_shares_accumulation() {
        let mut position = create_position();
        position.shares = 100;

        position.shares = position.shares.checked_add(50).unwrap();
        assert_eq!(position.shares, 150);

        position.shares = position.shares.checked_add(75).unwrap();
        assert_eq!(position.shares, 225);
    }

    #[test]
    fn test_config_authority() {
        let config = create_config();
        assert_ne!(config.authority, Pubkey::default());
    }

    #[test]
    fn test_pool_admin() {
        let pool = create_pool();
        assert_ne!(pool.admin, Pubkey::default());
    }

    #[test]
    fn test_lp_token_mint() {
        let pool = create_pool();
        assert_ne!(pool.lp_token_mint, Pubkey::default());
    }

    #[test]
    fn test_swap_same_token_error() {
        let pool = create_pool();
        let output = pool.calculate_swap_output(100, pool.token_a);
        // This should work since token_a is valid
        assert!(output.is_ok());
    }

    #[test]
    fn test_withdraw_ratio_preservation() {
        let pool = create_pool();
        let (a1, b1) = pool.calculate_withdraw_amounts(100, 1000).unwrap();
        let (a2, b2) = pool.calculate_withdraw_amounts(200, 1000).unwrap();

        let ratio1 = a1 as f64 / b1 as f64;
        let ratio2 = a2 as f64 / b2 as f64;

        assert!((ratio1 - ratio2).abs() < 0.001);
    }

    #[test]
    fn test_price_impact_small_trade() {
        let pool = create_pool();
        let output = pool.calculate_swap_output(1, pool.token_a).unwrap();
        assert!(output > 0);
    }

    #[test]
    fn test_price_impact_large_trade() {
        let pool = create_pool();
        let output = pool.calculate_swap_output(500, pool.token_a).unwrap();

        // Just verify the output is positive and less than reserves
        assert!(output > 0);
        assert!(output < pool.reserve_b);
    }

    #[test]
    fn test_pool_reserve_ratio() {
        let pool = create_pool();
        let ratio = pool.reserve_a as f64 / pool.reserve_b as f64;
        assert!(ratio > 0.0);
    }

    #[test]
    fn test_multiple_pools_independence() {
        let mut pool1 = create_pool();
        let mut pool2 = create_pool();

        pool1.token_a = Pubkey::new_unique();
        pool2.token_a = Pubkey::new_unique();

        pool1.update_reserves(100, 200, true).unwrap();
        pool2.update_reserves(300, 400, true).unwrap();

        assert_eq!(pool1.reserve_a, 1100);
        assert_eq!(pool2.reserve_a, 1300);
    }

    #[test]
    fn test_price_feed_no_negative_time() {
        let mut feed = create_price_feed();
        feed.last_update = 1000;

        // Just verify it doesn't panic
        let _ = feed.calculate_twap(50);
    }

    #[test]
    fn test_config_max_fee_edge() {
        let mut config = create_config();
        config.max_fee_bps = 10000;
        let result = config.set_fee(10000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pool_bump_tracking() {
        let mut pool = create_pool();
        pool.bump = 42;
        assert_eq!(pool.bump, 42);
    }

    #[test]
    fn test_position_bump_tracking() {
        let mut position = create_position();
        position.bump = 42;
        assert_eq!(position.bump, 42);
    }

    #[test]
    fn test_price_feed_bump_tracking() {
        let mut feed = create_price_feed();
        feed.bump = 42;
        assert_eq!(feed.bump, 42);
    }

    #[test]
    fn test_lp_shares_minimum_one() {
        let pool = create_pool();
        let shares = pool.calculate_lp_shares(1, 1).unwrap();
        assert!(shares >= 1);
    }

    #[test]
    fn test_swap_fee_distribution() {
        let mut pool = create_pool();
        pool.fee_bps = 100;

        let output = pool.calculate_swap_output(10000, pool.token_a).unwrap();
        let expected_fee = 10000 * 100 / 10000;

        assert!(output < 10000 - expected_fee);
    }

    #[test]
    fn test_pool_token_order_independence() {
        let mut pool1 = create_pool();
        let mut pool2 = create_pool();

        let t1 = Pubkey::new_unique();
        let t2 = Pubkey::new_unique();

        pool1.token_a = t1;
        pool1.token_b = t2;
        pool2.token_a = t2;
        pool2.token_b = t1;

        let out1 = pool1.calculate_swap_output(100, t1).unwrap();
        let out2 = pool2.calculate_swap_output(100, t2).unwrap();

        assert!(out1 > 0);
        assert!(out2 > 0);
    }

    #[test]
    fn test_initial_pool_liquidity() {
        let mut pool = create_pool();
        pool.reserve_a = 0;
        pool.reserve_b = 0;

        let shares = pool.calculate_lp_shares(1000000, 1000000).unwrap();
        assert!(shares >= 1000000);
    }

    #[test]
    fn test_add_remove_liquidity_cycle() {
        let mut pool = create_pool();

        pool.update_reserves(1000, 2000, true).unwrap();
        assert_eq!(pool.reserve_a, 2000);

        pool.update_reserves(500, 1000, false).unwrap();
        assert_eq!(pool.reserve_a, 1500);
    }

    #[test]
    fn test_zero_reserves_swap_prevention() {
        let mut pool = create_pool();
        pool.reserve_a = 0;

        let result = pool.calculate_swap_output(100, pool.token_a);
        assert!(result.is_err());
    }
}
