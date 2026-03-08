use std::result::Result as StdResult;

/// Result type for staking operations.
pub type StakingResult<T> = StdResult<T, StakingError>;

/// Staking-specific errors.
#[derive(Debug, Clone, Copy)]
pub enum StakingError {
    InvalidAmount,
    PoolInactive,
    MathOverflow,
    LockDurationNotMet,
    InvalidRewardRate,
    EntropyCheckFailed,
}

/// Information about a stake position.
#[derive(Clone, Copy)]
pub struct StakeInfo {
    pub amount: u64,
    pub last_reward_time: i64,
    pub pending_rewards: u64,
    pub locked_until: i64,
    pub entropy_score: u64, // v0.2.0: coherence of this stake
}

/// Trait for calculating staking rewards.
///
/// Can use simple time-based rewards or tensor-aware probability-based rewards.
pub trait StakingCalculator {
    /// Calculate pending rewards for a stake position.
    fn calculate_rewards(
        &self,
        stake_amount: u64,
        last_reward_time: i64,
        current_time: i64,
        reward_rate: u64,
    ) -> StakingResult<u64>;

    /// Calculate unlock probability based on entropy.
    ///
    /// Higher entropy (better optimization) = higher probability of early unlock.
    fn calculate_unlock_probability(
        &self,
        entropy_score: u64,
        s_max: u64,
        current_time: i64,
        locked_until: i64,
    ) -> u64;

    /// Calculate coherence bonus for early unlock.
    fn calculate_coherence_bonus(&self, entropy_score: u64, s_max: u64) -> u64;
}

/// Simple staking calculator (v0.1.x compatible).
///
/// Uses basic time-based reward calculation:
/// reward = (stake_amount * reward_rate * time_elapsed) / (365 * 24 * 3600 * 10000)
pub struct SimpleStakingCalculator;

impl SimpleStakingCalculator {
    pub fn new() -> Self {
        SimpleStakingCalculator
    }
}

impl Default for SimpleStakingCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl StakingCalculator for SimpleStakingCalculator {
    fn calculate_rewards(
        &self,
        stake_amount: u64,
        last_reward_time: i64,
        current_time: i64,
        reward_rate: u64,
    ) -> StakingResult<u64> {
        let time_elapsed = current_time
            .checked_sub(last_reward_time)
            .ok_or(StakingError::MathOverflow)? as u64;

        // reward = (amount * rate * time) / (365 * 24 * 3600 * 10000)
        // rate is in basis points (1 = 0.01% per second)
        let reward = (stake_amount as u128)
            .checked_mul(reward_rate as u128)
            .ok_or(StakingError::MathOverflow)?
            .checked_mul(time_elapsed as u128)
            .ok_or(StakingError::MathOverflow)?
            .checked_div(31_536_000_000_000u128) // 365 * 24 * 3600 * 10000
            .ok_or(StakingError::MathOverflow)? as u64;

        Ok(reward)
    }

    fn calculate_unlock_probability(
        &self,
        _entropy_score: u64,
        _s_max: u64,
        _current_time: i64,
        _locked_until: i64,
    ) -> u64 {
        0 // No early unlock in simple mode
    }

    fn calculate_coherence_bonus(&self, _entropy_score: u64, _s_max: u64) -> u64 {
        0 // No bonus in simple mode
    }
}

/// Tensor-aware staking calculator (v0.2.0).
///
/// Extends SimpleStakingCalculator with:
/// - Entropy-based unlock probability (P = tanh(S_A / S_max))
/// - Early unlock bonuses for high-coherence stakes
/// - Rewards weighted by entropy: R = base * (1 + entropy_weight)
///
/// Based on REALMS Part IV staking model:
/// - Stakes with high entropy have higher unlock probability
/// - Early unlock reduces lock period by entropy percentage
/// - Bonus rewards proportional to coherence
#[allow(dead_code)]
pub struct TensorAwareStakingCalculator {
    s_max: u64,
    entropy_weight: f64,
}

impl TensorAwareStakingCalculator {
    pub fn new(s_max: u64, entropy_weight: f64) -> Self {
        TensorAwareStakingCalculator {
            s_max,
            entropy_weight,
        }
    }

    /// Approximation of tanh for fixed-point arithmetic.
    fn tanh_approx(x: u64) -> u64 {
        const ONE: u64 = 1_000_000;

        if x >= 3 * ONE {
            return ONE;
        }
        if x == 0 {
            return 0;
        }

        // Polynomial: tanh(x) ≈ x - x³/3 + 2x⁵/15
        let x_cubed = (x / ONE) * (x / ONE) * (x / ONE) / ONE;
        let x_fifth = x_cubed * (x / ONE) * (x / ONE) / ONE;

        x.saturating_sub(x_cubed / 3)
            .saturating_add(2 * x_fifth / 15)
    }
}

impl StakingCalculator for TensorAwareStakingCalculator {
    fn calculate_rewards(
        &self,
        stake_amount: u64,
        last_reward_time: i64,
        current_time: i64,
        reward_rate: u64,
    ) -> StakingResult<u64> {
        // Base calculation same as simple
        let time_elapsed = current_time
            .checked_sub(last_reward_time)
            .ok_or(StakingError::MathOverflow)? as u64;

        let base_reward = (stake_amount as u128)
            .checked_mul(reward_rate as u128)
            .ok_or(StakingError::MathOverflow)?
            .checked_mul(time_elapsed as u128)
            .ok_or(StakingError::MathOverflow)?
            .checked_div(31_536_000_000_000u128)
            .ok_or(StakingError::MathOverflow)? as u64;

        // In full implementation, would apply entropy bonus here
        // For now, return base reward (entropy info provided separately)
        Ok(base_reward)
    }

    fn calculate_unlock_probability(
        &self,
        entropy_score: u64,
        _s_max: u64,
        _current_time: i64,
        _locked_until: i64,
    ) -> u64 {
        // P(unlock) = tanh(S_A / S_max)
        let normalized = (entropy_score * 1_000_000) / self.s_max.max(1);
        Self::tanh_approx(normalized)
    }

    fn calculate_coherence_bonus(&self, entropy_score: u64, _s_max: u64) -> u64 {
        // Bonus proportional to entropy: 0 at min, up to 10% at max
        let bonus_percent = (entropy_score as f64 / self.s_max as f64) * 0.1;
        (bonus_percent * 1_000_000.0) as u64
    }
}

/// Mock staking calculator for testing.
#[cfg(test)]
pub struct MockStakingCalculator {
    fixed_rewards: u64,
    unlock_probability: u64,
}

#[cfg(test)]
impl MockStakingCalculator {
    pub fn new(fixed_rewards: u64, unlock_probability: u64) -> Self {
        MockStakingCalculator {
            fixed_rewards,
            unlock_probability,
        }
    }
}

#[cfg(test)]
impl StakingCalculator for MockStakingCalculator {
    fn calculate_rewards(
        &self,
        _stake_amount: u64,
        _last_reward_time: i64,
        _current_time: i64,
        _reward_rate: u64,
    ) -> StakingResult<u64> {
        Ok(self.fixed_rewards)
    }

    fn calculate_unlock_probability(
        &self,
        _entropy_score: u64,
        _s_max: u64,
        _current_time: i64,
        _locked_until: i64,
    ) -> u64 {
        self.unlock_probability
    }

    fn calculate_coherence_bonus(&self, _entropy_score: u64, _s_max: u64) -> u64 {
        0
    }
}
