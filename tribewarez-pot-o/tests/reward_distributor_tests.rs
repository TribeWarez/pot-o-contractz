// Unit tests for RewardDistributor implementations
//
// Tests cover:
// 1. SimpleRewardDistributor - baseline v0.1.x behavior
// 2. TensorWeightedRewardDistributor - v0.2.0 enhancements
// 3. Penalty calculations
// 4. Pool reward distribution

use solana_sdk::pubkey::Pubkey;

// Mock implementations for testing (without Anchor dependencies)
mod mock_rewards {
    use solana_sdk::pubkey::Pubkey;

    #[derive(Clone, Copy)]
    pub struct RewardAllocation {
        pub base_reward: u64,
        pub bonus_reward: u64,
        pub total_reward: u64,
        pub multiplier: f64,
    }

    pub trait RewardDistributor {
        fn calculate_reward(
            &self,
            base_reward: u64,
            miner_reputation: u64,
            pool_id: Pubkey,
            device_type: u8,
        ) -> RewardAllocation;

        fn apply_penalty(&self, pending_rewards: u64, penalty_percent: u32) -> u64;

        fn distribute_pool_reward(
            &self,
            total_reward: u64,
            miner_shares: &[(Pubkey, u32)],
        ) -> Vec<(Pubkey, u64)>;
    }

    // Simple reward distributor
    pub struct SimpleRewardDistributor;

    impl SimpleRewardDistributor {
        pub fn new() -> Self {
            SimpleRewardDistributor
        }
    }

    impl RewardDistributor for SimpleRewardDistributor {
        fn calculate_reward(
            &self,
            base_reward: u64,
            _miner_reputation: u64,
            _pool_id: Pubkey,
            _device_type: u8,
        ) -> RewardAllocation {
            RewardAllocation {
                base_reward,
                bonus_reward: 0,
                total_reward: base_reward,
                multiplier: 1.0,
            }
        }

        fn apply_penalty(&self, pending_rewards: u64, penalty_percent: u32) -> u64 {
            let penalty = (pending_rewards as u128 * penalty_percent as u128 / 100) as u64;
            pending_rewards.saturating_sub(penalty)
        }

        fn distribute_pool_reward(
            &self,
            total_reward: u64,
            miner_shares: &[(Pubkey, u32)],
        ) -> Vec<(Pubkey, u64)> {
            if miner_shares.is_empty() {
                return vec![];
            }

            let total_weight: u64 = miner_shares.iter().map(|(_, w)| *w as u64).sum();
            if total_weight == 0 {
                return vec![];
            }

            let mut distribution: Vec<(Pubkey, u64)> = miner_shares
                .iter()
                .map(|(miner, weight)| {
                    let miner_reward =
                        (total_reward as u128 * *weight as u128 / total_weight as u128) as u64;
                    (*miner, miner_reward)
                })
                .collect();

            let distributed: u64 = distribution.iter().map(|(_, r)| *r).sum();
            if distributed < total_reward {
                if let Some((_, last_reward)) = distribution.last_mut() {
                    *last_reward += total_reward - distributed;
                }
            }

            distribution
        }
    }

    // Tensor-weighted distributor
    pub struct TensorWeightedRewardDistributor {
        s_max: u64,
        entropy_weight_factor: f64,
        coherence_multipliers: [f64; 4],
        pool_bonus_percent: u32,
    }

    impl TensorWeightedRewardDistributor {
        pub fn new(s_max: u64, entropy_weight_factor: f64) -> Self {
            TensorWeightedRewardDistributor {
                s_max,
                entropy_weight_factor,
                coherence_multipliers: [1.0, 1.1, 1.15, 0.9],
                pool_bonus_percent: 5,
            }
        }

        pub fn calculate_entropy_multiplier(&self, entropy_score: u64) -> f64 {
            1.0 + (entropy_score as f64 / self.s_max as f64) * self.entropy_weight_factor
        }

        pub fn get_coherence_multiplier(&self, device_type: u8) -> f64 {
            if (device_type as usize) < self.coherence_multipliers.len() {
                self.coherence_multipliers[device_type as usize]
            } else {
                1.0
            }
        }

        pub fn calculate_reputation_multiplier(&self, reputation: u64) -> f64 {
            if reputation < 100 {
                1.0
            } else {
                let bonus = ((reputation - 100) / 100).min(50);
                1.0 + (bonus as f64 / 100.0) * 0.5
            }
        }
    }

    impl RewardDistributor for TensorWeightedRewardDistributor {
        fn calculate_reward(
            &self,
            base_reward: u64,
            miner_reputation: u64,
            _pool_id: Pubkey,
            device_type: u8,
        ) -> RewardAllocation {
            let coherence_mult = self.get_coherence_multiplier(device_type);
            let reputation_mult = self.calculate_reputation_multiplier(miner_reputation);
            let combined_mult = (coherence_mult * reputation_mult).max(0.0);

            let total_reward = (base_reward as f64 * combined_mult) as u64;
            let bonus_reward = total_reward.saturating_sub(base_reward);

            RewardAllocation {
                base_reward,
                bonus_reward,
                total_reward,
                multiplier: combined_mult,
            }
        }

        fn apply_penalty(&self, pending_rewards: u64, penalty_percent: u32) -> u64 {
            let penalty = (pending_rewards as u128 * penalty_percent as u128 / 100) as u64;
            pending_rewards.saturating_sub(penalty)
        }

        fn distribute_pool_reward(
            &self,
            total_reward: u64,
            miner_shares: &[(Pubkey, u32)],
        ) -> Vec<(Pubkey, u64)> {
            if miner_shares.is_empty() {
                return vec![];
            }

            let total_weight: u64 = miner_shares.iter().map(|(_, w)| *w as u64).sum();
            if total_weight == 0 {
                return vec![];
            }

            // Apply pool bonus
            let bonus_multiplier = 1.0 + (self.pool_bonus_percent as f64 / 100.0);
            let adjusted_reward = (total_reward as f64 * bonus_multiplier) as u64;

            let mut distribution: Vec<(Pubkey, u64)> = miner_shares
                .iter()
                .map(|(miner, weight)| {
                    let miner_reward =
                        (adjusted_reward as u128 * *weight as u128 / total_weight as u128) as u64;
                    (*miner, miner_reward)
                })
                .collect();

            let distributed: u64 = distribution.iter().map(|(_, r)| *r).sum();
            if distributed < adjusted_reward {
                if let Some((_, last_reward)) = distribution.last_mut() {
                    *last_reward += adjusted_reward - distributed;
                }
            }

            distribution
        }
    }
}

use mock_rewards::*;

fn dummy_pubkey(seed: u8) -> Pubkey {
    Pubkey::new_from_array([seed; 32])
}

#[test]
fn test_simple_reward_distributor_fixed_reward() {
    let distributor = SimpleRewardDistributor::new();
    let pool = dummy_pubkey(1);

    let reward = distributor.calculate_reward(1000, 0, pool, 0);

    assert_eq!(reward.base_reward, 1000);
    assert_eq!(reward.bonus_reward, 0);
    assert_eq!(reward.total_reward, 1000);
    assert_eq!(reward.multiplier, 1.0);
}

#[test]
fn test_simple_reward_distributor_ignores_reputation() {
    let distributor = SimpleRewardDistributor::new();
    let pool = dummy_pubkey(1);

    let reward_low_rep = distributor.calculate_reward(1000, 0, pool, 0);
    let reward_high_rep = distributor.calculate_reward(1000, 5000, pool, 0);

    // SimpleRewardDistributor ignores reputation
    assert_eq!(reward_low_rep.total_reward, reward_high_rep.total_reward);
}

#[test]
fn test_simple_reward_distributor_ignores_device_type() {
    let distributor = SimpleRewardDistributor::new();
    let pool = dummy_pubkey(1);

    let reward_cpu = distributor.calculate_reward(1000, 0, pool, 0);
    let reward_asic = distributor.calculate_reward(1000, 0, pool, 2);

    // SimpleRewardDistributor ignores device type
    assert_eq!(reward_cpu.total_reward, reward_asic.total_reward);
}

#[test]
fn test_simple_reward_penalty() {
    let distributor = SimpleRewardDistributor::new();

    // 10% penalty on 1000 = 100 deducted
    let penalized = distributor.apply_penalty(1000, 10);
    assert_eq!(penalized, 900);

    // 50% penalty on 1000 = 500 deducted
    let penalized = distributor.apply_penalty(1000, 50);
    assert_eq!(penalized, 500);

    // 100% penalty
    let penalized = distributor.apply_penalty(1000, 100);
    assert_eq!(penalized, 0);
}

#[test]
fn test_simple_reward_pool_distribution() {
    let distributor = SimpleRewardDistributor::new();
    let miners = vec![
        (
            dummy_pubkey(1),
            10u32,
        ),
        (
            dummy_pubkey(2),
            20u32,
        ),
        (
            dummy_pubkey(3),
            30u32,
        ),
    ];

    let distribution = distributor.distribute_pool_reward(1000, &miners);

    assert_eq!(distribution.len(), 3);

    // Expected: 1000 * 10/(10+20+30) = 166, 1000 * 20/60 = 333, 1000 * 30/60 = 500
    // Due to integer division, might be slightly off
    let total: u64 = distribution.iter().map(|(_, r)| r).sum();
    assert_eq!(total, 1000);

    // Verify proportions (approximately)
    assert!(distribution[0].1 > 100); // First miner gets less
    assert!(distribution[1].1 > 250); // Second miner gets more
    assert!(distribution[2].1 > 450); // Third miner gets most
}

#[test]
fn test_tensor_reward_device_coherence_multiplier() {
    let distributor = TensorWeightedRewardDistributor::new(1_000_000, 0.5);
    let pool = dummy_pubkey(1);

    // Device types: CPU=0, GPU=1, ASIC=2, Mobile=3
    let reward_cpu = distributor.calculate_reward(1000, 0, pool, 0);
    let reward_gpu = distributor.calculate_reward(1000, 0, pool, 1);
    let reward_asic = distributor.calculate_reward(1000, 0, pool, 2);
    let reward_mobile = distributor.calculate_reward(1000, 0, pool, 3);

    // ASIC should have highest reward
    assert!(reward_asic.total_reward > reward_gpu.total_reward);
    assert!(reward_gpu.total_reward > reward_cpu.total_reward);

    // Mobile has lower coherence
    assert!(reward_mobile.total_reward < reward_cpu.total_reward);
}

#[test]
fn test_tensor_reward_reputation_scaling() {
    let distributor = TensorWeightedRewardDistributor::new(1_000_000, 0.5);
    let pool = dummy_pubkey(1);

    // Low reputation
    let reward_low = distributor.calculate_reward(1000, 50, pool, 1);

    // High reputation
    let reward_high = distributor.calculate_reward(1000, 500, pool, 1);

    // High reputation should earn more
    assert!(reward_high.total_reward > reward_low.total_reward);
}

#[test]
fn test_tensor_entropy_multiplier() {
    let distributor = TensorWeightedRewardDistributor::new(1_000_000, 0.5);

    // At 0 entropy: 1.0x
    let mult_0 = distributor.calculate_entropy_multiplier(0);
    assert!((mult_0 - 1.0).abs() < 0.01);

    // At max entropy (1_000_000): 1.0 + 0.5 = 1.5x
    let mult_max = distributor.calculate_entropy_multiplier(1_000_000);
    assert!((mult_max - 1.5).abs() < 0.01);

    // At half entropy: 1.0 + 0.25 = 1.25x
    let mult_half = distributor.calculate_entropy_multiplier(500_000);
    assert!((mult_half - 1.25).abs() < 0.01);
}

#[test]
fn test_tensor_reputation_multiplier() {
    let distributor = TensorWeightedRewardDistributor::new(1_000_000, 0.5);

    // Below 100: 1.0x
    assert!((distributor.calculate_reputation_multiplier(0) - 1.0).abs() < 0.01);
    assert!((distributor.calculate_reputation_multiplier(99) - 1.0).abs() < 0.01);

    // At 100: 1.0x (no bonus yet)
    assert!((distributor.calculate_reputation_multiplier(100) - 1.0).abs() < 0.01);

    // At 200: 1.0 + (1 * 0.5 / 100) = 1.005x
    let mult_200 = distributor.calculate_reputation_multiplier(200);
    assert!(mult_200 > 1.0);

    // At very high reputation: capped at 1.5x (50% bonus)
    let mult_high = distributor.calculate_reputation_multiplier(10000);
    assert!(mult_high <= 1.5);
}

#[test]
fn test_tensor_pool_reward_distribution_with_bonus() {
    let distributor = TensorWeightedRewardDistributor::new(1_000_000, 0.5);
    let miners = vec![
        (
            dummy_pubkey(1),
            10u32,
        ),
        (
            dummy_pubkey(2),
            20u32,
        ),
    ];

    let distribution = distributor.distribute_pool_reward(1000, &miners);

    assert_eq!(distribution.len(), 2);

    // With 5% bonus: total distributed = 1050
    let total: u64 = distribution.iter().map(|(_, r)| r).sum();
    assert_eq!(total, 1050);

    // Proportions should still be maintained (10:20 = 1:2)
    let ratio = distribution[1].1 as f64 / distribution[0].1 as f64;
    assert!((ratio - 2.0).abs() < 0.1);
}

#[test]
fn test_tensor_penalty() {
    let distributor = TensorWeightedRewardDistributor::new(1_000_000, 0.5);

    // 10% penalty
    let penalized = distributor.apply_penalty(1000, 10);
    assert_eq!(penalized, 900);

    // 100% penalty
    let penalized = distributor.apply_penalty(1000, 100);
    assert_eq!(penalized, 0);
}

#[test]
fn test_empty_pool_distribution() {
    let distributor = SimpleRewardDistributor::new();
    let distribution = distributor.distribute_pool_reward(1000, &[]);
    assert_eq!(distribution.len(), 0);
}

#[test]
fn test_zero_weight_pool_distribution() {
    let distributor = SimpleRewardDistributor::new();
    let miners = vec![
        (
            dummy_pubkey(1),
            0u32,
        ),
        (
            dummy_pubkey(2),
            0u32,
        ),
    ];
    let distribution = distributor.distribute_pool_reward(1000, &miners);
    assert_eq!(distribution.len(), 0);
}
