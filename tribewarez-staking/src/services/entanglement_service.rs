use anchor_lang::prelude::*;

/// Information about an entangled pool (group of related stakes).
#[derive(Clone, Copy)]
pub struct PoolEntanglement {
    pub pool_id: u32,
    pub miner_count: u32,
    pub total_entropy: u64,
    pub average_coherence: u64,
    pub efficiency_multiplier: f64, // 1.0 to 2.0
}

/// Trait for managing stake entanglement and pool relationships.
///
/// Entanglement in v0.2.0 allows groups of stakers to share tensor network
/// benefits through coherence coupling. Better coordination = higher rewards.
pub trait EntanglementService {
    /// Calculate efficiency bonus for a pool based on entanglement quality.
    ///
    /// Returns multiplier (1.0 = no bonus, 1.2 = 20% bonus)
    fn calculate_pool_efficiency(
        &self,
        pool_entropy: u64,
        miner_count: u32,
        max_pool_size: u32,
    ) -> f64;

    /// Join a stake to an existing entanglement pool.
    fn join_pool(
        &mut self,
        stake_id: Pubkey,
        pool_id: u32,
        entropy: u64,
        coherence: u64,
    ) -> Result<()>;

    /// Leave a stake from an entanglement pool.
    fn leave_pool(&mut self, stake_id: Pubkey, pool_id: u32) -> Result<()>;

    /// Calculate mutual entanglement benefit between two pools.
    ///
    /// Higher mutual information = better coupling = more stable rewards
    fn calculate_mutual_information(
        &self,
        pool_a_entropy: u64,
        pool_b_entropy: u64,
        shared_coherence: u64,
    ) -> u64;

    /// Check if pools should be merged (high mutual info suggests coupling).
    fn should_merge_pools(&self, pool_a_entropy: u64, pool_b_entropy: u64, threshold: u64) -> bool;
}

/// Simple entanglement service (v0.1.x compatible).
///
/// Tracks pools but doesn't apply entanglement bonuses.
pub struct SimpleEntanglementService;

impl SimpleEntanglementService {
    pub fn new() -> Self {
        SimpleEntanglementService
    }
}

impl Default for SimpleEntanglementService {
    fn default() -> Self {
        Self::new()
    }
}

impl EntanglementService for SimpleEntanglementService {
    fn calculate_pool_efficiency(
        &self,
        _pool_entropy: u64,
        _miner_count: u32,
        _max_pool_size: u32,
    ) -> f64 {
        1.0 // No efficiency bonus
    }

    fn join_pool(
        &mut self,
        _stake_id: Pubkey,
        _pool_id: u32,
        _entropy: u64,
        _coherence: u64,
    ) -> Result<()> {
        Ok(())
    }

    fn leave_pool(&mut self, _stake_id: Pubkey, _pool_id: u32) -> Result<()> {
        Ok(())
    }

    fn calculate_mutual_information(
        &self,
        _pool_a_entropy: u64,
        _pool_b_entropy: u64,
        _shared_coherence: u64,
    ) -> u64 {
        0
    }

    fn should_merge_pools(
        &self,
        _pool_a_entropy: u64,
        _pool_b_entropy: u64,
        _threshold: u64,
    ) -> bool {
        false
    }
}

/// Tensor-aware entanglement service (v0.2.0).
///
/// Implements full tensor network entanglement:
/// - Tracks pools as tensor network vertices
/// - Calculates mutual information between pools
/// - Applies efficiency multipliers based on coherence
/// - Suggests optimal pool merging
///
/// Based on REALMS Part IV coupling model.
pub struct TensorEntanglementService {
    s_max: u64,
    pools: Vec<PoolEntanglement>,
}

impl TensorEntanglementService {
    pub fn new(s_max: u64) -> Self {
        TensorEntanglementService {
            s_max,
            pools: Vec::new(),
        }
    }

    /// Calculate pool efficiency based on miner distribution and coherence.
    ///
    /// Efficiency bonus up to 20% when:
    /// - Pool is well-populated (>50% capacity)
    /// - High average coherence (>0.8)
    fn efficiency_formula(&self, pool_entropy: u64, miner_count: u32, max_pool_size: u32) -> f64 {
        if max_pool_size == 0 || pool_entropy == 0 {
            return 1.0;
        }

        // Population density bonus: 0 to 10%
        let fill_ratio = (miner_count as f64) / (max_pool_size as f64);
        let population_bonus = if fill_ratio > 0.5 {
            (fill_ratio - 0.5) * 0.2 // Up to 10% at full capacity
        } else {
            0.0
        };

        // Entropy-based coherence bonus: 0 to 10%
        let entropy_ratio = (pool_entropy as f64) / (self.s_max as f64);
        let coherence_bonus = entropy_ratio * 0.1;

        1.0 + (population_bonus + coherence_bonus).min(0.2) // Cap at 20%
    }
}

impl EntanglementService for TensorEntanglementService {
    fn calculate_pool_efficiency(
        &self,
        pool_entropy: u64,
        miner_count: u32,
        max_pool_size: u32,
    ) -> f64 {
        self.efficiency_formula(pool_entropy, miner_count, max_pool_size)
    }

    fn join_pool(
        &mut self,
        _stake_id: Pubkey,
        pool_id: u32,
        entropy: u64,
        coherence: u64,
    ) -> Result<()> {
        // Find pool and update entanglement
        if let Some(pool) = self.pools.iter_mut().find(|p| p.pool_id == pool_id) {
            pool.total_entropy = pool.total_entropy.saturating_add(entropy);
            pool.miner_count = pool.miner_count.saturating_add(1);
            // Recalculate average coherence
            pool.average_coherence = pool
                .total_entropy
                .checked_div(pool.miner_count as u64)
                .unwrap_or(coherence);
        }
        Ok(())
    }

    fn leave_pool(&mut self, _stake_id: Pubkey, pool_id: u32) -> Result<()> {
        // Find pool and decrement
        if let Some(pool) = self.pools.iter_mut().find(|p| p.pool_id == pool_id) {
            pool.miner_count = pool.miner_count.saturating_sub(1);
        }
        Ok(())
    }

    fn calculate_mutual_information(
        &self,
        pool_a_entropy: u64,
        pool_b_entropy: u64,
        shared_coherence: u64,
    ) -> u64 {
        // I(A:B) = S(A) + S(B) - S(A∪B)
        // Simplified: I ≈ shared_coherence * min(S_A, S_B) / S_max
        let _combined = pool_a_entropy.saturating_add(pool_b_entropy);
        let min_entropy = pool_a_entropy.min(pool_b_entropy);

        (shared_coherence as u128)
            .checked_mul(min_entropy as u128)
            .unwrap_or(0) as u64
            / self.s_max.max(1)
    }

    fn should_merge_pools(&self, pool_a_entropy: u64, pool_b_entropy: u64, threshold: u64) -> bool {
        // Merge if mutual information exceeds threshold
        let shared_coherence = 1_000_000u64; // Assume perfect coherence for merge decision
        let mutual_info =
            self.calculate_mutual_information(pool_a_entropy, pool_b_entropy, shared_coherence);
        mutual_info > threshold
    }
}

/// Mock entanglement service for testing.
#[cfg(test)]
pub struct MockEntanglementService {
    efficiency_multiplier: f64,
}

#[cfg(test)]
impl MockEntanglementService {
    pub fn new(efficiency: f64) -> Self {
        MockEntanglementService {
            efficiency_multiplier: efficiency,
        }
    }
}

#[cfg(test)]
impl EntanglementService for MockEntanglementService {
    fn calculate_pool_efficiency(
        &self,
        _pool_entropy: u64,
        _miner_count: u32,
        _max_pool_size: u32,
    ) -> f64 {
        self.efficiency_multiplier
    }

    fn join_pool(
        &mut self,
        _stake_id: Pubkey,
        _pool_id: u32,
        _entropy: u64,
        _coherence: u64,
    ) -> Result<()> {
        Ok(())
    }

    fn leave_pool(&mut self, _stake_id: Pubkey, _pool_id: u32) -> Result<()> {
        Ok(())
    }

    fn calculate_mutual_information(
        &self,
        _pool_a_entropy: u64,
        _pool_b_entropy: u64,
        _shared_coherence: u64,
    ) -> u64 {
        0
    }

    fn should_merge_pools(
        &self,
        _pool_a_entropy: u64,
        _pool_b_entropy: u64,
        _threshold: u64,
    ) -> bool {
        false
    }
}
