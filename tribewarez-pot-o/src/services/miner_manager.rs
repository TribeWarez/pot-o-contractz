use anchor_lang::prelude::*;

/// Miner information snapshot for management operations.
#[derive(Clone, Copy)]
pub struct MinerInfo {
    pub authority: Pubkey,
    pub device_type: u8,
    pub total_proofs: u64,
    pub total_rewards: u64,
    pub pending_rewards: u64,
    pub reputation_score: u64,
    pub last_proof_slot: u64,
    pub pool_id: Pubkey,
}

/// Trait for managing miner lifecycle and reputation.
pub trait MinerManager {
    /// Register a new miner with the given device type.
    fn register_miner(&self, authority: Pubkey, device_type: u8) -> Result<()>;

    /// Record a successful proof submission.
    fn record_proof(&self, miner: Pubkey, slot: u64, reward: u64) -> Result<()>;

    /// Update miner reputation based on various metrics.
    fn update_reputation(&self, miner: Pubkey, delta: i64) -> Result<()>;

    /// Get miner information (read-only query).
    fn get_miner_info(&self, miner: Pubkey) -> Result<Option<MinerInfo>>;

    /// Check if a miner is registered.
    fn is_miner_registered(&self, miner: Pubkey) -> Result<bool>;

    /// Calculate difficulty multiplier based on miner device type.
    /// Faster devices get higher difficulty to maintain constant block time.
    fn get_difficulty_multiplier(&self, device_type: u8) -> f64;
}

/// Standard miner manager (v0.1.x compatible).
///
/// Tracks miner registration, proof counts, and reputation scores.
/// Reputation is a simple counter incremented per proof.
pub struct StandardMinerManager;

impl StandardMinerManager {
    pub fn new() -> Self {
        StandardMinerManager
    }
}

impl Default for StandardMinerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MinerManager for StandardMinerManager {
    fn register_miner(&self, _authority: Pubkey, _device_type: u8) -> Result<()> {
        // In actual implementation, this would write to on-chain state
        // For now, the instruction handler manages state directly
        Ok(())
    }

    fn record_proof(&self, _miner: Pubkey, _slot: u64, _reward: u64) -> Result<()> {
        // In actual implementation, this would update miner stats
        // For now, the instruction handler manages state directly
        Ok(())
    }

    fn update_reputation(&self, _miner: Pubkey, _delta: i64) -> Result<()> {
        // Reputation update logic
        Ok(())
    }

    fn get_miner_info(&self, _miner: Pubkey) -> Result<Option<MinerInfo>> {
        // Query miner data (instruction context provides this)
        Ok(None)
    }

    fn is_miner_registered(&self, _miner: Pubkey) -> Result<bool> {
        Ok(false)
    }

    fn get_difficulty_multiplier(&self, device_type: u8) -> f64 {
        // Device type difficulty scaling (standard v0.1.x)
        match device_type {
            0 => 1.0, // CPU: baseline
            1 => 2.0, // GPU: 2x faster
            2 => 4.0, // ASIC: 4x faster
            3 => 0.5, // Mobile: 0.5x (throttled)
            _ => 1.0, // Unknown: baseline
        }
    }
}

/// Tensor-aware miner manager (v0.2.0).
///
/// Extends StandardMinerManager with:
/// - Entanglement pool tracking (which miners are bonded together)
/// - Entropy-weighted reputation (high-entropy proofs reward more)
/// - Device type coherence factors (some devices maintain entanglement better)
///
/// Based on REALMS Part IV coupling model:
/// - Miners form entangled pools that maximize mutual information
/// - Coherence penalty for devices with higher decoherence rates
/// - Reputation weighted by proof entropy (not just count)
#[allow(dead_code)]
pub struct TensorAwareMinerManager {
    max_pool_size: u32,
    entropy_weight: f64,         // How much entropy affects reward multiplier
    coherence_factors: [f64; 4], // Device coherence preservation (0-1)
}

impl TensorAwareMinerManager {
    pub fn new(max_pool_size: u32, entropy_weight: f64) -> Self {
        TensorAwareMinerManager {
            max_pool_size,
            entropy_weight,
            // Coherence factors by device type
            // ASIC > GPU > CPU > Mobile (in ability to maintain quantum coherence)
            coherence_factors: [0.6, 0.8, 1.0, 0.4],
        }
    }

    /// Calculate reputation multiplier based on entropy and device type.
    ///
    /// High entropy proofs from good-coherence devices get boosted reputation.
    pub fn calculate_reputation_multiplier(
        &self,
        entropy_score: u64, // 0 - 1_000_000
        device_type: u8,
    ) -> f64 {
        let s_max = 1_000_000u64;

        // Entropy multiplier: 1.0 at median, up to 2.0 at max entropy
        let entropy_mult = 1.0 + (entropy_score as f64 / s_max as f64);

        // Coherence factor for device
        let coherence = if (device_type as usize) < self.coherence_factors.len() {
            self.coherence_factors[device_type as usize]
        } else {
            1.0
        };

        entropy_mult * coherence
    }

    /// Recommend pool assignment based on device coherence and current pools.
    ///
    /// Miners with similar coherence factors should be pooled together
    /// to maximize entanglement quality.
    pub fn recommend_pool_for_device(&self, device_type: u8) -> u8 {
        // Pool 0: High coherence (ASIC, GPU)
        // Pool 1: Medium coherence (CPU)
        // Pool 2: Low coherence (Mobile)
        match device_type {
            1..=2 => 0, // GPU, ASIC -> high coherence pool
            0 => 1,     // CPU -> medium coherence pool
            3 => 2,     // Mobile -> low coherence pool
            _ => 0,     // Unknown -> high coherence pool
        }
    }
}

impl MinerManager for TensorAwareMinerManager {
    fn register_miner(&self, _authority: Pubkey, _device_type: u8) -> Result<()> {
        Ok(())
    }

    fn record_proof(&self, _miner: Pubkey, _slot: u64, _reward: u64) -> Result<()> {
        Ok(())
    }

    fn update_reputation(&self, _miner: Pubkey, _delta: i64) -> Result<()> {
        Ok(())
    }

    fn get_miner_info(&self, _miner: Pubkey) -> Result<Option<MinerInfo>> {
        Ok(None)
    }

    fn is_miner_registered(&self, _miner: Pubkey) -> Result<bool> {
        Ok(false)
    }

    fn get_difficulty_multiplier(&self, device_type: u8) -> f64 {
        // In tensor-aware mode, penalize devices with poor coherence
        let base_multiplier = match device_type {
            0 => 1.0, // CPU
            1 => 2.0, // GPU
            2 => 4.0, // ASIC
            3 => 0.5, // Mobile
            _ => 1.0,
        };

        let coherence = if (device_type as usize) < self.coherence_factors.len() {
            self.coherence_factors[device_type as usize]
        } else {
            1.0
        };

        // Apply coherence penalty: poor coherence = higher difficulty
        base_multiplier / coherence
    }
}

/// Mock miner manager for testing.
#[cfg(test)]
pub struct MockMinerManager {
    registered_miners: Vec<Pubkey>,
}

#[cfg(test)]
impl MockMinerManager {
    pub fn new() -> Self {
        MockMinerManager {
            registered_miners: Vec::new(),
        }
    }

    pub fn with_registered(mut self, miner: Pubkey) -> Self {
        self.registered_miners.push(miner);
        self
    }
}

#[cfg(test)]
impl MinerManager for MockMinerManager {
    fn register_miner(&self, _authority: Pubkey, _device_type: u8) -> Result<()> {
        Ok(())
    }

    fn record_proof(&self, _miner: Pubkey, _slot: u64, _reward: u64) -> Result<()> {
        Ok(())
    }

    fn update_reputation(&self, _miner: Pubkey, _delta: i64) -> Result<()> {
        Ok(())
    }

    fn get_miner_info(&self, _miner: Pubkey) -> Result<Option<MinerInfo>> {
        Ok(None)
    }

    fn is_miner_registered(&self, miner: Pubkey) -> Result<bool> {
        Ok(self.registered_miners.contains(&miner))
    }

    fn get_difficulty_multiplier(&self, _device_type: u8) -> f64 {
        1.0 // No scaling in mock
    }
}
