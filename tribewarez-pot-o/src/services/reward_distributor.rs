use anchor_lang::prelude::*;

/// Reward distribution result containing amount and any metadata.
#[derive(Clone, Copy)]
pub struct RewardAllocation {
    pub base_reward: u64,
    pub bonus_reward: u64,
    pub total_reward: u64,
    pub multiplier: f64, // For tracking what multiplier was applied
}

/// Trait for calculating and distributing mining rewards.
pub trait RewardDistributor {
    /// Calculate reward for a successfully validated proof.
    ///
    /// Parameters:
    /// - `base_reward`: Configured reward per proof
    /// - `miner_reputation`: Miner's reputation score (may affect bonus)
    /// - `pool_id`: Pool the miner belongs to (may affect distribution)
    /// - `device_type`: Miner's device type
    ///
    /// Returns:
    /// - RewardAllocation with base, bonus, and total reward amounts
    fn calculate_reward(
        &self,
        base_reward: u64,
        miner_reputation: u64,
        pool_id: Pubkey,
        device_type: u8,
    ) -> RewardAllocation;

    /// Apply a penalty to pending rewards (e.g., for slashing).
    ///
    /// Returns new pending reward amount after penalty.
    fn apply_penalty(&self, pending_rewards: u64, penalty_percent: u32) -> u64;

    /// Distribute rewards to a pool of miners (for shared mining pools).
    ///
    /// Takes total pool reward and splits among participants based on
    /// their contribution metrics.
    fn distribute_pool_reward(
        &self,
        total_reward: u64,
        miner_shares: &[(Pubkey, u32)], // (miner, share_weight)
    ) -> Vec<(Pubkey, u64)>;
}

/// Simple reward distributor (v0.1.x compatible).
///
/// All miners receive the same fixed reward per validated proof.
/// No bonuses or penalties applied.
pub struct SimpleRewardDistributor;

impl SimpleRewardDistributor {
    pub fn new() -> Self {
        SimpleRewardDistributor
    }
}

impl Default for SimpleRewardDistributor {
    fn default() -> Self {
        Self::new()
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

        miner_shares
            .iter()
            .map(|(miner, weight)| {
                let miner_reward =
                    (total_reward as u128 * *weight as u128 / total_weight as u128) as u64;
                (*miner, miner_reward)
            })
            .collect()
    }
}

/// Tensor-weighted reward distributor (v0.2.0).
///
/// Extends SimpleRewardDistributor with:
/// - Entropy-based bonuses (high entropy proofs earn more)
/// - Device coherence multipliers (coherent devices earn bonus)
/// - Pool cooperation bonuses (miners in large entangled pools earn more)
/// - Reputation scaling (higher reputation = higher rewards)
///
/// Based on REALMS Part IV staking model:
/// - Rewards proportional to entropy contribution: R = base_reward * (1 + S_A / S_max)
/// - Coherence bonus: devices maintaining entanglement get 10-20% bonus
/// - Pool cooperation: larger coherent pools get efficiency bonus
///
/// Reward formula:
/// total_reward = base * (1 + entropy_weight) * coherence_multiplier * pool_bonus
pub struct TensorWeightedRewardDistributor {
    s_max: u64,                      // Maximum entropy (1e6 scale)
    entropy_weight_factor: f64,      // Entropy contribution weight (0.5 = 50% bonus at max entropy)
    coherence_multipliers: [f64; 4], // Device coherence reward bonus
    pool_bonus_percent: u32,         // Bonus for miners in pools (e.g., 5%)
}

impl TensorWeightedRewardDistributor {
    pub fn new(s_max: u64, entropy_weight_factor: f64) -> Self {
        TensorWeightedRewardDistributor {
            s_max,
            entropy_weight_factor,
            // Device coherence bonuses: ASIC > GPU > CPU > Mobile
            coherence_multipliers: [1.0, 1.1, 1.15, 0.9],
            pool_bonus_percent: 5, // 5% bonus for being in a pool
        }
    }

    /// Calculate entropy bonus multiplier.
    ///
    /// At 0 entropy: 1.0x
    /// At max entropy: 1.0 + entropy_weight_factor
    pub fn calculate_entropy_multiplier(&self, entropy_score: u64) -> f64 {
        1.0 + (entropy_score as f64 / self.s_max as f64) * self.entropy_weight_factor
    }

    /// Calculate coherence bonus for device type.
    pub fn get_coherence_multiplier(&self, device_type: u8) -> f64 {
        if (device_type as usize) < self.coherence_multipliers.len() {
            self.coherence_multipliers[device_type as usize]
        } else {
            1.0
        }
    }

    /// Calculate reputation scaling factor.
    ///
    /// Reputation >= 100 gets bonus, capped at 50% additional reward.
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
        // For now, we only apply coherence multiplier
        // Full entropy calculation requires TensorPoolService context
        // which should be used in the actual instruction

        let coherence_mult = self.get_coherence_multiplier(device_type);
        let reputation_mult = self.calculate_reputation_multiplier(miner_reputation);
        let combined_mult = coherence_mult * reputation_mult;

        let bonus_reward = (base_reward as f64 * (combined_mult - 1.0)) as u64;
        let total_reward = base_reward + bonus_reward;

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

        // Apply pool bonus: each miner gets their share + pool_bonus_percent
        let pool_bonus = (total_reward as f64 * self.pool_bonus_percent as f64 / 100.0) as u64;
        let reward_after_bonus = total_reward.saturating_add(pool_bonus);

        miner_shares
            .iter()
            .map(|(miner, weight)| {
                let miner_reward =
                    (reward_after_bonus as u128 * *weight as u128 / total_weight as u128) as u64;
                (*miner, miner_reward)
            })
            .collect()
    }
}

/// Mock reward distributor for testing.
#[cfg(test)]
pub struct MockRewardDistributor {
    fixed_reward: u64,
}

#[cfg(test)]
impl MockRewardDistributor {
    pub fn new(fixed_reward: u64) -> Self {
        MockRewardDistributor { fixed_reward }
    }
}

#[cfg(test)]
impl RewardDistributor for MockRewardDistributor {
    fn calculate_reward(
        &self,
        _base_reward: u64,
        _miner_reputation: u64,
        _pool_id: Pubkey,
        _device_type: u8,
    ) -> RewardAllocation {
        RewardAllocation {
            base_reward: self.fixed_reward,
            bonus_reward: 0,
            total_reward: self.fixed_reward,
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

        miner_shares
            .iter()
            .map(|(miner, weight)| {
                let miner_reward =
                    (total_reward as u128 * *weight as u128 / total_weight as u128) as u64;
                (*miner, miner_reward)
            })
            .collect()
    }
}
