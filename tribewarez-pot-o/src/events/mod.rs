use anchor_lang::prelude::*;

/// Emitted when a miner registers for the first time.
#[event]
pub struct MinerRegistered {
    pub authority: Pubkey,
    pub device_type: u8,
    pub timestamp: i64,
}

/// Emitted when a proof is successfully submitted and validated.
#[event]
pub struct ProofSubmitted {
    pub miner: Pubkey,
    pub challenge_id: [u8; 32],
    pub mml_score: u64,
    pub slot: u64,
    pub timestamp: i64,
    pub entropy_score: u64,
    pub is_tensor_aware: bool,
}

/// Emitted when rewards are distributed to a miner.
#[event]
pub struct RewardDistributed {
    pub miner: Pubkey,
    pub base_reward: u64,
    pub bonus_reward: u64,
    pub total_reward: u64,
    pub multiplier: u64, // f64 as u64 (1.0x = 1_000_000)
    pub timestamp: i64,
}

/// Emitted when entropy state of the network is updated.
/// (NEW in v0.2.0)
#[event]
pub struct EntropyStateUpdated {
    pub network_entropy: u64,    // S_network in 1e6 scale
    pub max_entropy: u64,        // S_max in 1e6 scale
    pub total_miners: u32,       // Number of miners in network
    pub active_pools: u32,       // Number of entanglement pools
    pub average_coherence: u64,  // Average device coherence (1e6 scale)
    pub slot: u64,
    pub timestamp: i64,
}
