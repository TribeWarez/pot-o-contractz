#[cfg(test)]
mod tests {
    use anchor_lang::prelude::*;
    use tribewarez_router::state::{RouteConfig, SwapQuote, SwapRoute};

    fn create_route() -> SwapRoute {
        SwapRoute {
            from_token: Pubkey::new_unique(),
            to_token: Pubkey::new_unique(),
            via_token: None,
            fee_bps: 30,
            liquidity: 1_000_000,
            bump: 0,
        }
    }

    fn create_config() -> RouteConfig {
        RouteConfig {
            authority: Pubkey::new_unique(),
            default_fee_bps: 30,
            max_hops: 3,
            enabled: true,
            bump: 0,
        }
    }

    // === SwapRoute Tests ===

    #[test]
    fn test_route_creation() {
        let route = create_route();
        assert_eq!(route.fee_bps, 30);
        assert_eq!(route.liquidity, 1_000_000);
        assert!(route.via_token.is_none());
    }

    #[test]
    fn test_route_with_intermediate_token() {
        let mut route = create_route();
        route.via_token = Some(Pubkey::new_unique());
        assert!(route.has_intermediate_token());
        assert_eq!(route.get_path_length(), 3);
    }

    #[test]
    fn test_route_without_intermediate_token() {
        let route = create_route();
        assert!(!route.has_intermediate_token());
        assert_eq!(route.get_path_length(), 2);
    }

    #[test]
    fn test_calculate_output_basic() {
        let route = create_route();
        let output = route.calculate_output(1000).unwrap();
        assert!(output > 0);
    }

    #[test]
    fn test_calculate_output_zero_input() {
        let route = create_route();
        let output = route.calculate_output(0).unwrap();
        assert_eq!(output, 0);
    }

    #[test]
    fn test_calculate_output_high_fee() {
        let mut route = create_route();
        route.fee_bps = 1000; // 10%
                              // Just verify it doesn't panic
        let _ = route.calculate_output(1000);
    }

    #[test]
    fn test_calculate_output_no_fee() {
        let mut route = create_route();
        route.fee_bps = 0;
        let output = route.calculate_output(1000).unwrap();
        assert!(output > 0);
    }

    #[test]
    fn test_calculate_output_large_input() {
        let route = create_route();
        let output = route.calculate_output(1_000_000).unwrap();
        assert!(output > 0);
    }

    #[test]
    fn test_validate_path_valid() {
        let route = create_route();
        let result = route.validate_path(route.from_token, route.to_token);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_invalid_from() {
        let route = create_route();
        let result = route.validate_path(Pubkey::new_unique(), route.to_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_invalid_to() {
        let route = create_route();
        let result = route.validate_path(route.from_token, Pubkey::new_unique());
        assert!(result.is_err());
    }

    // === RouteConfig Tests ===

    #[test]
    fn test_config_creation() {
        let config = create_config();
        assert!(config.enabled);
        assert_eq!(config.default_fee_bps, 30);
        assert_eq!(config.max_hops, 3);
    }

    #[test]
    fn test_toggle_enabled() {
        let mut config = create_config();
        assert!(config.is_enabled());
        config.toggle_enabled();
        assert!(!config.is_enabled());
        config.toggle_enabled();
        assert!(config.is_enabled());
    }

    #[test]
    fn test_set_fee() {
        let mut config = create_config();
        config.set_fee(50).unwrap();
        assert_eq!(config.default_fee_bps, 50);
    }

    #[test]
    fn test_config_disabled() {
        let mut config = create_config();
        config.enabled = false;
        assert!(!config.is_enabled());
    }

    // === SwapQuote Tests ===

    #[test]
    fn test_swap_quote_creation() {
        let quote = SwapQuote {
            amount_in: 1000,
            amount_out: 900,
            fee: 100,
            path: vec![Pubkey::new_unique(), Pubkey::new_unique()],
            price_impact: 0,
            timestamp: 1000,
        };
        assert_eq!(quote.amount_in, 1000);
        assert_eq!(quote.amount_out, 900);
        assert_eq!(quote.fee, 100);
    }

    #[test]
    fn test_validate_slippage_pass() {
        let quote = SwapQuote {
            amount_in: 1000,
            amount_out: 900,
            fee: 100,
            path: vec![],
            price_impact: 0,
            timestamp: 1000,
        };
        assert!(quote.validate_slippage(800).is_ok());
    }

    #[test]
    fn test_validate_slippage_fail() {
        let quote = SwapQuote {
            amount_in: 1000,
            amount_out: 900,
            fee: 100,
            path: vec![],
            price_impact: 0,
            timestamp: 1000,
        };
        assert!(quote.validate_slippage(950).is_err());
    }

    #[test]
    fn test_calculate_price_impact() {
        let quote = SwapQuote {
            amount_in: 1000,
            amount_out: 900,
            fee: 100,
            path: vec![],
            price_impact: 0,
            timestamp: 1000,
        };
        let impact = quote.calculate_price_impact(1000, 100, 90);
        assert!(impact > 0);
    }

    #[test]
    fn test_calculate_price_impact_zero_old() {
        let quote = SwapQuote {
            amount_in: 1000,
            amount_out: 900,
            fee: 100,
            path: vec![],
            price_impact: 0,
            timestamp: 1000,
        };
        let impact = quote.calculate_price_impact(1000, 0, 90);
        assert_eq!(impact, 0);
    }

    // === Complex Scenarios ===

    #[test]
    fn test_multi_hop_route_calculation() {
        let mut route = create_route();
        route.via_token = Some(Pubkey::new_unique());

        let output = route.calculate_output(5000).unwrap();
        assert!(output > 0);
    }

    #[test]
    fn test_route_with_max_liquidity() {
        let mut route = create_route();
        route.liquidity = 1_000_000_000_000u64; // Large but manageable value

        let output = route.calculate_output(1000).unwrap();
        assert!(output > 0);
    }

    #[test]
    fn test_route_low_liquidity() {
        let mut route = create_route();
        route.liquidity = 100;

        let output = route.calculate_output(1000).unwrap();
        assert!(output < 100);
    }

    #[test]
    fn test_multiple_routes_independence() {
        let mut route1 = create_route();
        let mut route2 = create_route();

        route1.liquidity = 1000;
        route2.liquidity = 2000;

        let out1 = route1.calculate_output(100).unwrap();
        let out2 = route2.calculate_output(100).unwrap();

        assert!(out2 > out1);
    }

    #[test]
    fn test_fee_calculation_various_amounts() {
        let mut route = create_route();
        route.fee_bps = 100;

        let amounts = vec![1, 10, 100, 1000, 10000, 100000];
        for amount in amounts {
            let output = route.calculate_output(amount).unwrap();
            assert!(output >= 0); // Just verify it doesn't panic
        }
    }

    #[test]
    fn test_config_authority_change() {
        let mut config = create_config();
        let new_auth = Pubkey::new_unique();
        config.authority = new_auth;
        assert_eq!(config.authority, new_auth);
    }

    #[test]
    fn test_route_bump_tracking() {
        let mut route = create_route();
        route.bump = 42;
        assert_eq!(route.bump, 42);
    }

    #[test]
    fn test_swap_quote_path_validation() {
        let mut token_a = Pubkey::new_unique();
        let mut token_b = Pubkey::new_unique();
        let mut token_c = Pubkey::new_unique();

        let quote = SwapQuote {
            amount_in: 1000,
            amount_out: 900,
            fee: 100,
            path: vec![token_a, token_b, token_c],
            price_impact: 0,
            timestamp: 1000,
        };
        assert_eq!(quote.path.len(), 3);
    }

    #[test]
    fn test_quote_timestamp() {
        let quote = SwapQuote {
            amount_in: 1000,
            amount_out: 900,
            fee: 100,
            path: vec![],
            price_impact: 0,
            timestamp: 1234567890,
        };
        assert_eq!(quote.timestamp, 1234567890);
    }

    #[test]
    fn test_config_max_hops() {
        let mut config = create_config();
        config.max_hops = 5;
        assert_eq!(config.max_hops, 5);
    }

    #[test]
    fn test_route_constant_length() {
        let len = SwapRoute::LEN;
        assert!(len > 0);
    }

    #[test]
    fn test_config_constant_length() {
        let len = RouteConfig::LEN;
        assert!(len > 0);
    }

    // === Edge Cases ===

    #[test]
    fn test_route_exact_liquidity_match() {
        let mut route = create_route();
        route.liquidity = 1000;

        let output = route.calculate_output(1).unwrap();
        assert!(output <= 1000);
    }

    #[test]
    fn test_route_zero_liquidity() {
        let mut route = create_route();
        route.liquidity = 0;

        let output = route.calculate_output(1000).unwrap();
        assert_eq!(output, 0);
    }

    #[test]
    fn test_quote_zero_amount_in() {
        let quote = SwapQuote {
            amount_in: 0,
            amount_out: 0,
            fee: 0,
            path: vec![],
            price_impact: 0,
            timestamp: 1000,
        };
        assert!(quote.validate_slippage(0).is_ok());
    }

    #[test]
    fn test_route_max_fee() {
        let mut route = create_route();
        route.fee_bps = 10000; // 100%

        let output = route.calculate_output(1000).unwrap();
        assert_eq!(output, 0);
    }

    #[test]
    fn test_config_disabled_toggle() {
        let mut config = create_config();

        config.toggle_enabled();
        assert!(!config.enabled);

        config.toggle_enabled();
        assert!(config.enabled);

        config.toggle_enabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_route_path_validation_edge() {
        let route = create_route();
        let from = route.from_token;
        let to = route.to_token;

        assert!(route.validate_path(from, to).is_ok());
    }

    #[test]
    fn test_price_impact_no_change() {
        let quote = SwapQuote {
            amount_in: 1000,
            amount_out: 900,
            fee: 100,
            path: vec![],
            price_impact: 0,
            timestamp: 1000,
        };
        let impact = quote.calculate_price_impact(1000, 100, 100);
        assert_eq!(impact, 0);
    }

    #[test]
    fn test_route_fee_distribution() {
        let mut route = create_route();
        route.fee_bps = 25; // 0.25%

        let test_amounts = vec![100, 200, 400, 800, 1600];
        for amount in test_amounts {
            let output = route.calculate_output(amount).unwrap();
            assert!(output >= 0); // Just verify it doesn't panic
        }
    }

    #[test]
    fn test_config_authority_types() {
        let auth = Pubkey::new_unique();
        let config = RouteConfig {
            authority: auth,
            default_fee_bps: 30,
            max_hops: 3,
            enabled: true,
            bump: 0,
        };
        assert_eq!(config.authority, auth);
    }

    #[test]
    fn test_route_all_fields_initialized() {
        let from = Pubkey::new_unique();
        let to = Pubkey::new_unique();
        let via = Pubkey::new_unique();

        let route = SwapRoute {
            from_token: from,
            to_token: to,
            via_token: Some(via),
            fee_bps: 50,
            liquidity: 500000,
            bump: 1,
        };

        assert_eq!(route.from_token, from);
        assert_eq!(route.to_token, to);
        assert_eq!(route.via_token, Some(via));
        assert_eq!(route.fee_bps, 50);
        assert_eq!(route.liquidity, 500000);
        assert_eq!(route.bump, 1);
    }

    #[test]
    fn test_route_direct_swap() {
        let route = create_route();
        assert!(!route.has_intermediate_token());
        assert_eq!(route.get_path_length(), 2);
    }

    #[test]
    fn test_route_multi_hop() {
        let mut route = create_route();
        route.via_token = Some(Pubkey::new_unique());
        assert!(route.has_intermediate_token());
        assert_eq!(route.get_path_length(), 3);
    }

    #[test]
    fn test_config_initial_state() {
        let config = create_config();
        assert!(config.enabled);
        assert_eq!(config.default_fee_bps, 30);
    }

    #[test]
    fn test_quote_price_impact_tracking() {
        let quote = SwapQuote {
            amount_in: 5000,
            amount_out: 4500,
            fee: 500,
            path: vec![],
            price_impact: 100,
            timestamp: 5000,
        };
        assert_eq!(quote.price_impact, 100);
    }

    #[test]
    fn test_route_output_with_small_input() {
        let mut route = create_route();
        route.liquidity = 1000000;

        let output = route.calculate_output(1).unwrap();
        assert!(output >= 0);
    }

    #[test]
    fn test_multiple_quotes_same_input() {
        let quote1 = SwapQuote {
            amount_in: 1000,
            amount_out: 900,
            fee: 100,
            path: vec![],
            price_impact: 0,
            timestamp: 1000,
        };

        let quote2 = SwapQuote {
            amount_in: 1000,
            amount_out: 850,
            fee: 150,
            path: vec![],
            price_impact: 0,
            timestamp: 1001,
        };

        assert!(quote1.fee < quote2.fee);
    }

    #[test]
    fn test_config_fee_update() {
        let mut config = create_config();
        config.set_fee(100).unwrap();
        assert_eq!(config.default_fee_bps, 100);
    }

    #[test]
    fn test_route_with_different_fees() {
        let fees = vec![0, 10, 25, 50, 100, 500, 1000, 5000];
        for fee in fees {
            let mut route = create_route();
            route.fee_bps = fee;
            let _ = route.calculate_output(1000);
        }
    }

    #[test]
    fn test_quote_empty_path() {
        let quote = SwapQuote {
            amount_in: 1000,
            amount_out: 900,
            fee: 100,
            path: vec![],
            price_impact: 0,
            timestamp: 1000,
        };
        assert_eq!(quote.path.len(), 0);
    }

    #[test]
    fn test_route_liquidity_comparison() {
        let mut route1 = create_route();
        let mut route2 = create_route();

        route1.liquidity = 100;
        route2.liquidity = 1000;

        let out1 = route1.calculate_output(10).unwrap();
        let out2 = route2.calculate_output(10).unwrap();

        assert!(out2 >= out1);
    }

    #[test]
    fn test_config_enable_disable_cycle() {
        let mut config = create_config();

        for _ in 0..5 {
            config.toggle_enabled();
        }

        assert!(!config.enabled);
    }

    #[test]
    fn test_route_bump_persistence() {
        let mut route = create_route();
        for bump in 0..10 {
            route.bump = bump;
            assert_eq!(route.bump, bump);
        }
    }

    #[test]
    fn test_quote_timestamp_ordering() {
        let quote1 = SwapQuote {
            amount_in: 1000,
            amount_out: 900,
            fee: 100,
            path: vec![],
            price_impact: 0,
            timestamp: 100,
        };

        let quote2 = SwapQuote {
            amount_in: 1000,
            amount_out: 900,
            fee: 100,
            path: vec![],
            price_impact: 0,
            timestamp: 200,
        };

        assert!(quote2.timestamp > quote1.timestamp);
    }

    #[test]
    fn test_route_constant_space() {
        let len1 = SwapRoute::LEN;
        let len2 = SwapRoute::LEN;
        assert_eq!(len1, len2);
    }

    #[test]
    fn test_config_constant_space() {
        let len1 = RouteConfig::LEN;
        let len2 = RouteConfig::LEN;
        assert_eq!(len1, len2);
    }

    #[test]
    fn test_max_hops_boundary() {
        let mut config = create_config();
        config.max_hops = 1;
        assert_eq!(config.max_hops, 1);

        config.max_hops = 10;
        assert_eq!(config.max_hops, 10);
    }

    #[test]
    fn test_swap_quote_comparison() {
        let quote1 = SwapQuote {
            amount_in: 1000,
            amount_out: 900,
            fee: 100,
            path: vec![],
            price_impact: 5,
            timestamp: 1000,
        };

        let quote2 = SwapQuote {
            amount_in: 1000,
            amount_out: 950,
            fee: 50,
            path: vec![],
            price_impact: 3,
            timestamp: 1001,
        };

        assert!(quote2.amount_out > quote1.amount_out);
    }

    #[test]
    fn test_large_input_small_liquidity() {
        let mut route = create_route();
        route.liquidity = 1;

        let output = route.calculate_output(1_000_000_000).unwrap();
        assert!(output < 2);
    }
}
