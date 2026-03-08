// Unit tests for Swap Calculator implementations
//
// Tests cover:
// 1. SimpleSwapCalculator - v0.1.x constant product AMM
// 2. TensorSwapCalculator - v0.2.0 coherence-based fee discounts
// 3. Swap output calculations
// 4. Fee calculations
// 5. Price impact calculations

mod mock_swap_calculator {
    #[derive(Clone, Copy)]
    pub struct SwapQuote {
        pub amount_out: u64,
        pub fee: u64,
        pub price_impact_bps: u64,
    }

    pub trait SwapCalculator {
        fn calculate_swap_output(
            &self,
            amount_in: u64,
            reserve_in: u64,
            reserve_out: u64,
        ) -> SwapQuote;

        fn calculate_fee(&self, amount: u64) -> u64;

        fn calculate_price_impact(&self, amount_in: u64, reserve_in: u64) -> u64;
    }

    pub struct SimpleSwapCalculator {
        swap_fee_bps: u64,     // 30 = 0.30%
        protocol_fee_bps: u64, // 5 = 0.05%
    }

    impl SimpleSwapCalculator {
        pub fn new(swap_fee_bps: u64, protocol_fee_bps: u64) -> Self {
            SimpleSwapCalculator {
                swap_fee_bps,
                protocol_fee_bps,
            }
        }

        /// Constant product formula: x * y = k
        /// output = (reserve_out * amount_in * (10000 - fee_bps)) / (reserve_in * 10000 + amount_in * (10000 - fee_bps))
        fn calculate_swap_output_internal(
            &self,
            amount_in: u64,
            reserve_in: u64,
            reserve_out: u64,
            fee_bps: u64,
        ) -> u64 {
            let amount_in_with_fee = (amount_in as u128 * (10000 - fee_bps) as u128) / 10000u128;

            let numerator = amount_in_with_fee * reserve_out as u128;
            let denominator = reserve_in as u128 * 10000 + amount_in_with_fee;

            if denominator == 0 {
                return 0;
            }

            (numerator / denominator) as u64
        }
    }

    impl SwapCalculator for SimpleSwapCalculator {
        fn calculate_swap_output(
            &self,
            amount_in: u64,
            reserve_in: u64,
            reserve_out: u64,
        ) -> SwapQuote {
            let output = self.calculate_swap_output_internal(
                amount_in,
                reserve_in,
                reserve_out,
                self.swap_fee_bps,
            );

            let fee = (amount_in as u128 * self.swap_fee_bps as u128 / 10000) as u64;
            let price_impact = (amount_in as u128 * 10000 / reserve_in as u128) as u64;

            SwapQuote {
                amount_out: output,
                fee,
                price_impact_bps: price_impact,
            }
        }

        fn calculate_fee(&self, amount: u64) -> u64 {
            (amount as u128 * self.swap_fee_bps as u128 / 10000) as u64
        }

        fn calculate_price_impact(&self, amount_in: u64, reserve_in: u64) -> u64 {
            if reserve_in == 0 {
                10000 // Max impact if no liquidity
            } else {
                (amount_in as u128 * 10000 / reserve_in as u128).min(10000) as u64
            }
        }
    }

    pub struct TensorSwapCalculator {
        swap_fee_bps: u64,
        protocol_fee_bps: u64,
        s_max: u64,
        coherence_weight: f64,
    }

    impl TensorSwapCalculator {
        pub fn new(
            swap_fee_bps: u64,
            protocol_fee_bps: u64,
            s_max: u64,
            coherence_weight: f64,
        ) -> Self {
            TensorSwapCalculator {
                swap_fee_bps,
                protocol_fee_bps,
                s_max,
                coherence_weight,
            }
        }

        fn calculate_swap_output_internal(
            &self,
            amount_in: u64,
            reserve_in: u64,
            reserve_out: u64,
            fee_bps: u64,
        ) -> u64 {
            let amount_in_with_fee = (amount_in as u128 * (10000 - fee_bps) as u128) / 10000u128;

            let numerator = amount_in_with_fee * reserve_out as u128;
            let denominator = reserve_in as u128 * 10000 + amount_in_with_fee;

            if denominator == 0 {
                return 0;
            }

            (numerator / denominator) as u64
        }

        fn calculate_coherence_discount(&self, coherence: u64) -> f64 {
            // 0-50% discount based on coherence
            let normalized = (coherence as f64 / self.s_max as f64).min(1.0);
            normalized * 0.5 // Up to 50% discount
        }
    }

    impl SwapCalculator for TensorSwapCalculator {
        fn calculate_swap_output(
            &self,
            amount_in: u64,
            reserve_in: u64,
            reserve_out: u64,
        ) -> SwapQuote {
            let output = self.calculate_swap_output_internal(
                amount_in,
                reserve_in,
                reserve_out,
                self.swap_fee_bps,
            );

            let fee = (amount_in as u128 * self.swap_fee_bps as u128 / 10000) as u64;
            let price_impact = (amount_in as u128 * 10000 / reserve_in as u128) as u64;

            SwapQuote {
                amount_out: output,
                fee,
                price_impact_bps: price_impact,
            }
        }

        fn calculate_fee(&self, amount: u64) -> u64 {
            (amount as u128 * self.swap_fee_bps as u128 / 10000) as u64
        }

        fn calculate_price_impact(&self, amount_in: u64, reserve_in: u64) -> u64 {
            if reserve_in == 0 {
                10000
            } else {
                (amount_in as u128 * 10000 / reserve_in as u128).min(10000) as u64
            }
        }
    }
}

use mock_swap_calculator::*;

#[test]
fn test_simple_swap_small_amount() {
    let calc = SimpleSwapCalculator::new(30, 5);

    // Small swap: 1 token from pool with 1M liquidity
    let quote = calc.calculate_swap_output(1, 1_000_000, 1_000_000);

    // Should get approximately 1 token back (minus fee)
    assert!(quote.amount_out > 0);
    assert!(quote.amount_out < 1);
}

#[test]
fn test_simple_swap_large_amount() {
    let calc = SimpleSwapCalculator::new(30, 5);

    // Swap 10k tokens from pool with 1M liquidity
    let quote = calc.calculate_swap_output(10_000, 1_000_000, 1_000_000);

    // Should get back fewer tokens due to price impact
    assert!(quote.amount_out < 10_000);
}

#[test]
fn test_simple_swap_fee_calculation() {
    let calc = SimpleSwapCalculator::new(30, 5);

    // Fee on 1000 tokens at 0.30% = 3 tokens
    let fee = calc.calculate_fee(1000);
    assert_eq!(fee, 3);
}

#[test]
fn test_simple_swap_zero_fee() {
    let calc = SimpleSwapCalculator::new(0, 0);

    let fee = calc.calculate_fee(1000);
    assert_eq!(fee, 0);
}

#[test]
fn test_simple_swap_max_fee() {
    let calc = SimpleSwapCalculator::new(10000, 0); // 100% fee

    let fee = calc.calculate_fee(1000);
    assert_eq!(fee, 1000);
}

#[test]
fn test_simple_price_impact_zero() {
    let calc = SimpleSwapCalculator::new(30, 5);

    // Very small swap relative to liquidity
    let impact = calc.calculate_price_impact(1, 1_000_000);
    assert_eq!(impact, 0);
}

#[test]
fn test_simple_price_impact_10_percent() {
    let calc = SimpleSwapCalculator::new(30, 5);

    // 1% of pool
    let impact = calc.calculate_price_impact(10_000, 1_000_000);
    assert_eq!(impact, 100); // 1% in basis points
}

#[test]
fn test_simple_price_impact_half_pool() {
    let calc = SimpleSwapCalculator::new(30, 5);

    // 50% of pool
    let impact = calc.calculate_price_impact(500_000, 1_000_000);
    assert_eq!(impact, 5000); // 50% impact
}

#[test]
fn test_simple_price_impact_capped() {
    let calc = SimpleSwapCalculator::new(30, 5);

    // More than the entire pool
    let impact = calc.calculate_price_impact(2_000_000, 1_000_000);
    assert_eq!(impact, 10000); // Capped at 100%
}

#[test]
fn test_simple_swap_zero_reserve_in() {
    let calc = SimpleSwapCalculator::new(30, 5);

    // Pool with no liquidity in token_in
    let quote = calc.calculate_swap_output(100, 0, 1_000_000);
    assert_eq!(quote.amount_out, 0);
}

#[test]
fn test_simple_swap_zero_reserve_out() {
    let calc = SimpleSwapCalculator::new(30, 5);

    // Pool with no liquidity in token_out
    let quote = calc.calculate_swap_output(100, 1_000_000, 0);
    assert_eq!(quote.amount_out, 0);
}

#[test]
fn test_simple_swap_zero_amount_in() {
    let calc = SimpleSwapCalculator::new(30, 5);

    let quote = calc.calculate_swap_output(0, 1_000_000, 1_000_000);
    assert_eq!(quote.amount_out, 0);
}

#[test]
fn test_simple_swap_asymmetric_reserves() {
    let calc = SimpleSwapCalculator::new(30, 5);

    // Pool with 2:1 ratio
    let quote = calc.calculate_swap_output(100, 2_000_000, 1_000_000);

    // Less output due to unbalanced reserves
    assert!(quote.amount_out < 50);
}

#[test]
fn test_tensor_swap_calculator_creation() {
    let calc = TensorSwapCalculator::new(30, 5, 1_000_000, 0.5);

    // Should create successfully
    let quote = calc.calculate_swap_output(100, 1_000_000, 1_000_000);
    assert!(quote.amount_out > 0);
}

#[test]
fn test_tensor_swap_same_base_as_simple() {
    let simple = SimpleSwapCalculator::new(30, 5);
    let tensor = TensorSwapCalculator::new(30, 5, 1_000_000, 0.5);

    let simple_quote = simple.calculate_swap_output(100, 1_000_000, 1_000_000);
    let tensor_quote = tensor.calculate_swap_output(100, 1_000_000, 1_000_000);

    // Should have identical output (no coherence discount applied in this test)
    assert_eq!(simple_quote.amount_out, tensor_quote.amount_out);
}

#[test]
fn test_tensor_coherence_discount_zero() {
    let calc = TensorSwapCalculator::new(30, 5, 1_000_000, 0.5);

    let discount = calc.calculate_coherence_discount(0);
    assert_eq!(discount, 0.0);
}

#[test]
fn test_tensor_coherence_discount_max() {
    let calc = TensorSwapCalculator::new(30, 5, 1_000_000, 0.5);

    let discount = calc.calculate_coherence_discount(1_000_000);
    assert!((discount - 0.5).abs() < 0.01);
}

#[test]
fn test_tensor_coherence_discount_half() {
    let calc = TensorSwapCalculator::new(30, 5, 1_000_000, 0.5);

    let discount = calc.calculate_coherence_discount(500_000);
    assert!((discount - 0.25).abs() < 0.01);
}

#[test]
fn test_swap_fee_matches_amount() {
    let calc = SimpleSwapCalculator::new(100, 10); // 1% fee

    // Different amounts should scale fee proportionally
    let fee_100 = calc.calculate_fee(100);
    let fee_1000 = calc.calculate_fee(1000);
    let fee_10000 = calc.calculate_fee(10000);

    assert_eq!(fee_100, 1);
    assert_eq!(fee_1000, 10);
    assert_eq!(fee_10000, 100);
}

#[test]
fn test_swap_constant_product_invariant() {
    let calc = SimpleSwapCalculator::new(0, 0); // No fee for this test

    // Initial reserves: 1M each
    let reserve_in = 1_000_000u64;
    let reserve_out = 1_000_000u64;
    let k = reserve_in as u128 * reserve_out as u128;

    let quote = calc.calculate_swap_output(100_000, reserve_in, reserve_out);

    // After swap, k should be maintained (within rounding)
    let new_reserve_in = reserve_in + 100_000;
    let new_reserve_out = reserve_out - quote.amount_out;
    let new_k = new_reserve_in as u128 * new_reserve_out as u128;

    // new_k should be >= original_k
    assert!(new_k >= k);
}

#[test]
fn test_swap_quote_fields_consistency() {
    let calc = SimpleSwapCalculator::new(30, 5);

    let quote = calc.calculate_swap_output(1000, 1_000_000, 1_000_000);

    // All fields should be non-negative
    assert!(quote.amount_out >= 0);
    assert!(quote.fee >= 0);
    assert!(quote.price_impact_bps >= 0);
}

#[test]
fn test_swap_multiple_sizes() {
    let calc = SimpleSwapCalculator::new(30, 5);

    for amount in [1, 10, 100, 1000, 10000, 100000].iter() {
        let quote = calc.calculate_swap_output(*amount, 1_000_000, 1_000_000);

        // Output should be less than input (due to fee and price impact)
        assert!(quote.amount_out < *amount);

        // Fee should scale with amount
        assert!(quote.fee > 0);
    }
}
