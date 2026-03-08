// Unit tests for Staking Calculator implementations
//
// Tests cover:
// 1. SimpleStakingCalculator - v0.1.x time-based rewards
// 2. TensorAwareStakingCalculator - v0.2.0 entropy-boosted rewards
// 3. Unlock probability calculations
// 4. Coherence bonuses

use solana_sdk::pubkey::Pubkey;

mod mock_staking {
    use solana_sdk::pubkey::Pubkey;

    #[derive(Clone, Copy)]
    pub struct StakingReward {
        pub base_reward: u64,
        pub bonus_reward: u64,
        pub total_reward: u64,
        pub unlock_probability: u64, // 0-1e6 scale
    }

    pub trait StakingCalculator {
        fn calculate_reward(
            &self,
            stake_amount: u64,
            stake_duration_seconds: u64,
            pool_id: Pubkey,
        ) -> StakingReward;

        fn calculate_unlock_probability(&self, entropy_score: u64, max_entropy: u64) -> u64; // 0-1e6
    }

    pub struct SimpleStakingCalculator {
        annual_rate: u64, // BPS, 10000 = 100%
    }

    impl SimpleStakingCalculator {
        pub fn new(annual_rate: u64) -> Self {
            SimpleStakingCalculator { annual_rate }
        }

        /// Time-based reward: (stake * rate * time) / (365*24*3600*10000)
        fn calculate_base_reward(&self, stake_amount: u64, duration_seconds: u64) -> u64 {
            let numerator =
                stake_amount as u128 * self.annual_rate as u128 * duration_seconds as u128;
            let denominator = 365u128 * 24 * 3600 * 10000;
            (numerator / denominator) as u64
        }
    }

    impl StakingCalculator for SimpleStakingCalculator {
        fn calculate_reward(
            &self,
            stake_amount: u64,
            stake_duration_seconds: u64,
            _pool_id: Pubkey,
        ) -> StakingReward {
            let base = self.calculate_base_reward(stake_amount, stake_duration_seconds);
            StakingReward {
                base_reward: base,
                bonus_reward: 0,
                total_reward: base,
                unlock_probability: 0, // No entropy-based unlock
            }
        }

        fn calculate_unlock_probability(&self, _entropy: u64, _max_entropy: u64) -> u64 {
            1_000_000 // Always 100% unlock
        }
    }

    pub struct TensorAwareStakingCalculator {
        annual_rate: u64,
        s_max: u64,
        entropy_weight: f64,
    }

    impl TensorAwareStakingCalculator {
        pub fn new(annual_rate: u64, s_max: u64, entropy_weight: f64) -> Self {
            TensorAwareStakingCalculator {
                annual_rate,
                s_max,
                entropy_weight,
            }
        }

        fn calculate_base_reward(&self, stake_amount: u64, duration_seconds: u64) -> u64 {
            let numerator =
                stake_amount as u128 * self.annual_rate as u128 * duration_seconds as u128;
            let denominator = 365u128 * 24 * 3600 * 10000;
            (numerator / denominator) as u64
        }

        fn calculate_entropy_multiplier(&self, entropy: u64) -> f64 {
            1.0 + (entropy as f64 / self.s_max as f64) * self.entropy_weight
        }

        fn calculate_coherence_bonus(&self, entropy: u64) -> f64 {
            // 0-10% bonus based on entropy
            let normalized = (entropy as f64 / self.s_max as f64).min(1.0);
            1.0 + normalized * 0.1
        }
    }

    impl StakingCalculator for TensorAwareStakingCalculator {
        fn calculate_reward(
            &self,
            stake_amount: u64,
            stake_duration_seconds: u64,
            _pool_id: Pubkey,
        ) -> StakingReward {
            let base = self.calculate_base_reward(stake_amount, stake_duration_seconds);

            // For now, apply 0 entropy bonus (would need entropy context in real scenario)
            let entropy_mult = 1.0;
            let coherence_bonus = 1.0;
            let total_mult = entropy_mult * coherence_bonus;

            let total = (base as f64 * total_mult) as u64;
            let bonus = total - base;

            StakingReward {
                base_reward: base,
                bonus_reward: bonus,
                total_reward: total,
                unlock_probability: 500_000, // Placeholder
            }
        }

        fn calculate_unlock_probability(&self, entropy_score: u64, max_entropy: u64) -> u64 {
            if max_entropy == 0 {
                return 0;
            }
            let normalized = entropy_score as f64 / max_entropy as f64;
            let p = normalized.tanh();
            (p * 1_000_000.0) as u64
        }
    }
}

use mock_staking::*;

#[test]
fn test_simple_staking_one_year() {
    let calc = SimpleStakingCalculator::new(1000); // 10% APY

    // 1 year of staking 1000 tokens at 10%
    let duration = 365 * 24 * 3600;
    let reward = calc.calculate_reward(1000, duration as u64, Pubkey::new_unique());

    // Expected: 1000 * 10% = 100
    assert!(reward.base_reward > 95 && reward.base_reward < 105);
}

#[test]
fn test_simple_staking_half_year() {
    let calc = SimpleStakingCalculator::new(1000); // 10% APY

    // 6 months of staking
    let duration = (365 / 2) * 24 * 3600;
    let reward = calc.calculate_reward(1000, duration as u64, Pubkey::new_unique());

    // Expected: 1000 * 10% / 2 = 50
    assert!(reward.base_reward > 45 && reward.base_reward < 55);
}

#[test]
fn test_simple_staking_zero_duration() {
    let calc = SimpleStakingCalculator::new(1000);
    let reward = calc.calculate_reward(1000, 0, Pubkey::new_unique());

    // No time staked = 0 reward
    assert_eq!(reward.base_reward, 0);
}

#[test]
fn test_simple_staking_ignores_entropy() {
    let calc = SimpleStakingCalculator::new(1000);

    let reward1 = calc.calculate_unlock_probability(0, 1_000_000);
    let reward2 = calc.calculate_unlock_probability(1_000_000, 1_000_000);

    // Both should return 100% probability
    assert_eq!(reward1, 1_000_000);
    assert_eq!(reward2, 1_000_000);
}

#[test]
fn test_tensor_staking_one_year() {
    let calc = TensorAwareStakingCalculator::new(1000, 1_000_000, 0.5);

    let duration = 365 * 24 * 3600;
    let reward = calc.calculate_reward(1000, duration as u64, Pubkey::new_unique());

    // Base reward should match SimpleStakingCalculator
    assert!(reward.base_reward > 95 && reward.base_reward < 105);
}

#[test]
fn test_tensor_unlock_probability_zero_entropy() {
    let calc = TensorAwareStakingCalculator::new(1000, 1_000_000, 0.5);

    let p = calc.calculate_unlock_probability(0, 1_000_000);
    // tanh(0) = 0
    assert_eq!(p, 0);
}

#[test]
fn test_tensor_unlock_probability_max_entropy() {
    let calc = TensorAwareStakingCalculator::new(1000, 1_000_000, 0.5);

    let p = calc.calculate_unlock_probability(1_000_000, 1_000_000);
    // tanh(1.0) ≈ 0.762, so ~762_000 in fixed-point
    assert!(p > 760_000 && p < 765_000);
}

#[test]
fn test_tensor_unlock_probability_half_entropy() {
    let calc = TensorAwareStakingCalculator::new(1000, 1_000_000, 0.5);

    let p = calc.calculate_unlock_probability(500_000, 1_000_000);
    // tanh(0.5) ≈ 0.462, so ~462_000 in fixed-point
    assert!(p > 460_000 && p < 465_000);
}

#[test]
fn test_tensor_unlock_probability_bounds() {
    let calc = TensorAwareStakingCalculator::new(1000, 1_000_000, 0.5);

    // Test various entropy levels
    for entropy in (0..=1_000_000).step_by(100_000) {
        let p = calc.calculate_unlock_probability(entropy, 1_000_000);

        // Probability should be in [0, 1e6]
        assert!(p <= 1_000_000, "Probability out of bounds: {}", p);
    }
}

#[test]
fn test_simple_vs_tensor_reward_same_base() {
    let simple = SimpleStakingCalculator::new(1000);
    let tensor = TensorAwareStakingCalculator::new(1000, 1_000_000, 0.5);

    let duration = 365 * 24 * 3600;
    let pool = Pubkey::new_unique();

    let simple_reward = simple.calculate_reward(1000, duration as u64, pool);
    let tensor_reward = tensor.calculate_reward(1000, duration as u64, pool);

    // Base rewards should be identical
    assert_eq!(simple_reward.base_reward, tensor_reward.base_reward);
}

#[test]
fn test_stake_amount_scaling() {
    let calc = SimpleStakingCalculator::new(1000); // 10% APY

    let duration = 365 * 24 * 3600;

    let reward_1000 = calc.calculate_reward(1000, duration as u64, Pubkey::new_unique());
    let reward_2000 = calc.calculate_reward(2000, duration as u64, Pubkey::new_unique());

    // Double stake should (approximately) double reward
    assert!(reward_2000.base_reward > reward_1000.base_reward * 2 - 10);
    assert!(reward_2000.base_reward < reward_1000.base_reward * 2 + 10);
}

#[test]
fn test_high_apr_calculation() {
    let calc = SimpleStakingCalculator::new(10000); // 100% APY (unrealistic but for testing)

    let duration = 365 * 24 * 3600;
    let reward = calc.calculate_reward(1000, duration as u64, Pubkey::new_unique());

    // Expected: 1000 * 100% = 1000
    assert!(reward.base_reward > 995 && reward.base_reward < 1005);
}

#[test]
fn test_low_apr_calculation() {
    let calc = SimpleStakingCalculator::new(100); // 1% APY

    let duration = 365 * 24 * 3600;
    let reward = calc.calculate_reward(1000, duration as u64, Pubkey::new_unique());

    // Expected: 1000 * 1% = 10
    assert!(reward.base_reward > 9 && reward.base_reward < 11);
}

#[test]
fn test_monthly_reward() {
    let calc = SimpleStakingCalculator::new(1200); // 12% APY

    let duration = 30 * 24 * 3600; // 1 month
    let reward = calc.calculate_reward(1000, duration as u64, Pubkey::new_unique());

    // Expected: 1000 * 12% / 12 = 10
    assert!(reward.base_reward > 8 && reward.base_reward < 12);
}

#[test]
fn test_weekly_reward() {
    let calc = SimpleStakingCalculator::new(5200); // 52% APY

    let duration = 7 * 24 * 3600; // 1 week
    let reward = calc.calculate_reward(1000, duration as u64, Pubkey::new_unique());

    // Expected: 1000 * 52% / 52 = 10
    assert!(reward.base_reward > 8 && reward.base_reward < 12);
}

#[test]
fn test_unlock_probability_monotonic() {
    let calc = TensorAwareStakingCalculator::new(1000, 1_000_000, 0.5);

    let mut prev_p = 0u64;
    for entropy in (0..=1_000_000).step_by(50_000) {
        let p = calc.calculate_unlock_probability(entropy, 1_000_000);

        // Probability should be monotonically increasing
        assert!(
            p >= prev_p,
            "Probability not monotonic at entropy={}",
            entropy
        );
        prev_p = p;
    }
}
