//! # tribewarez-pot-o
//!
//! Proof of Tensor Optimizations (PoT-O) on-chain program for Solana.
//!
//! This crate implements the PoT-O blockchain validation system, which combines proof-of-work
//! concepts with tensor network analysis to optimize mining rewards. It manages miner accounts,
//! validates cryptographic proofs submitted by off-chain validators, and distributes rewards
//! with optional tensor-aware enhancements.
//!
//! ## Core Features
//!
//! - **Proof Validation**: Validates mining proofs submitted by the off-chain PoT-O validator
//! - **Miner Management**: Creates and maintains miner accounts with proof history tracking
//! - **Reward Distribution**: Distributes mining rewards with support for tensor-weighted bonuses
//! - **Difficulty Adjustment**: Automatically adjusts mining difficulty based on network conditions
//! - **Tensor Network Support**: v0.2.0 feature for enhanced reward calculations using tensor metrics
//!
//! ## Key Instructions
//!
//! - `initialize`: Set up the PoT-O configuration (admin-only)
//! - `register_miner`: Register a new miner account
//! - `submit_proof`: Submit a mining proof for validation and reward distribution
//! - `update_pool_config`: Update configuration parameters (admin-only)
//! - `create_tensor_pool`: Create a tensor network pool for enhanced rewards (admin-only)
//!
//! ## Events
//!
//! This program emits events for proof submissions, reward distributions, and configuration changes.
//! See the [`events`] module for detailed event documentation.
//!
//! ## Services
//!
//! The [`services`] module provides trait implementations for proof validation and reward distribution,
//! supporting both legacy (v0.1.x) and tensor-aware (v0.2.0) configurations.

use anchor_lang::prelude::*;

// Module declarations
pub mod events;
pub mod services;

// Re-export services for use in instructions
use events::{ProofSubmitted, RewardDistributed};
use services::{ProofData, ServiceRegistry};

declare_id!("1PoToVa1idator11111111111111111111111111111");

/// PoT-O: Proof of Tensor Optimizations on-chain program.
/// Validates mining proofs submitted by the off-chain PoT-O validator,
/// manages miner accounts, distributes rewards, and adjusts difficulty.
#[program]
pub mod tribewarez_pot_o {
    use super::*;

    /// Initialize the PoT-O config. Called once by the admin.
    ///
    /// Sets up both v0.1.x and v0.2.0 configuration. Tensor network features
    /// are disabled by default but can be enabled via update_pool_config.
    pub fn initialize(ctx: Context<Initialize>, params: InitParams) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.difficulty = params.difficulty;
        config.mml_threshold = params.mml_threshold;
        config.path_distance_max = params.path_distance_max;
        config.reward_per_proof = params.reward_per_proof;
        config.total_proofs = 0;
        config.pool_type = params.pool_type;
        config.swap_program_id = params.swap_program_id;
        config.bump = ctx.bumps.config;

        // Initialize tensor network fields (v0.2.0)
        config.tensor_enabled = 0; // Disabled by default, enable via update_pool_config
        config.s_max = 1_000_000; // 1e6 scale
        config.bond_dimension = 2; // Default: 2 bonds per vertex
        config.max_pool_size = 128; // Max 128 miners per entanglement pool
        config.entropy_weight_factor = 500_000; // 0.5x entropy weight
        config.network_entropy = 0;
        config.total_miners = 0;
        config.active_pools = 0;
        config.average_coherence = 0;

        Ok(())
    }

    /// Register a new miner.
    ///
    /// Initializes miner account with device type and computes initial coherence
    /// based on device type if tensor network is enabled.
    pub fn register_miner(ctx: Context<RegisterMiner>, device_type: u8) -> Result<()> {
        let miner = &mut ctx.accounts.miner_account;
        miner.authority = ctx.accounts.authority.key();
        miner.device_type = device_type;
        miner.total_proofs = 0;
        miner.total_rewards = 0;
        miner.pending_rewards = 0;
        miner.reputation_score = 0;
        miner.last_proof_slot = 0;
        miner.pool_id = Pubkey::default();
        miner.bump = ctx.bumps.miner_account;

        // Initialize tensor fields (v0.2.0)
        miner.vertex_id = 0; // Will be assigned by pool service
        miner.entropy_score = 0; // Will be calculated on first proof
        miner.coherence = compute_device_coherence(device_type);
        miner.last_entropy_update = 0;
        miner.entanglement_count = 0;
        miner.pool_generation = 0;
        miner.unlock_probability = 0; // Will be calculated from entropy

        // Emit event
        let clock = Clock::get()?;
        emit!(events::MinerRegistered {
            authority: ctx.accounts.authority.key(),
            device_type,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Submit a PoT-O proof. Validates on-chain using ServiceRegistry and distributes rewards.
    ///
    /// This instruction delegates validation and reward calculation to the service layer,
    /// enabling both legacy (v0.1.x) and tensor-aware (v0.2.0) modes based on config.
    ///
    /// Flow:
    /// 1. Instantiate ServiceRegistry based on config.tensor_enabled
    /// 2. Use ProofValidator to validate proof (standard or tensor-aware)
    /// 3. Use RewardDistributor to calculate rewards with bonuses
    /// 4. Update miner stats and emit events
    pub fn submit_proof(ctx: Context<SubmitProof>, params: ProofParams) -> Result<()> {
        let config = &ctx.accounts.config;
        let clock = Clock::get()?;

        // Instantiate appropriate ServiceRegistry based on config
        let registry = if config.tensor_enabled != 0 {
            // Tensor-aware mode (v0.2.0)
            ServiceRegistry::new_tensor_aware(
                config.s_max,
                config.bond_dimension,
                (config.entropy_weight_factor as f64) / 1_000_000.0,
                config.max_pool_size,
            )
        } else {
            // Legacy mode (v0.1.x)
            ServiceRegistry::new_legacy()
        };

        // Prepare proof data for validation
        let proof_data = ProofData {
            challenge_id: params.challenge_id,
            challenge_slot: params.challenge_slot,
            tensor_result_hash: params.tensor_result_hash,
            mml_score: params.mml_score,
            path_signature: params.path_signature,
            path_distance: params.path_distance,
            computation_nonce: params.computation_nonce,
            computation_hash: params.computation_hash,
        };

        // 1. Validate proof using service
        let validator = registry.proof_validator();
        let validated_proof = validator
            .validate(
                &proof_data,
                clock.slot,
                config.mml_threshold,
                config.path_distance_max,
            )
            .map_err(|e| match e {
                services::ValidationError::ChallengeExpired => PotOError::ChallengeExpired,
                services::ValidationError::MmlThresholdNotMet => PotOError::MmlThresholdNotMet,
                services::ValidationError::PathDistanceTooLarge => PotOError::PathDistanceTooLarge,
                services::ValidationError::InvalidComputationHash => {
                    PotOError::InvalidComputationHash
                }
                _ => PotOError::InvalidComputationHash, // Other errors map to generic
            })?;

        // 2. Calculate reward using service
        let miner_account = &mut ctx.accounts.miner_account;
        let distributor = registry.reward_distributor();
        let reward_allocation = distributor.calculate_reward(
            config.reward_per_proof,
            miner_account.reputation_score,
            miner_account.pool_id,
            miner_account.device_type,
        );

        // 3. Record proof with tensor data
        let proof_record = &mut ctx.accounts.proof_record;
        proof_record.miner = ctx.accounts.miner.key();
        proof_record.challenge_id = params.challenge_id;
        proof_record.mml_score = params.mml_score;
        proof_record.path_signature = params.path_signature;
        proof_record.slot = clock.slot;
        proof_record.timestamp = clock.unix_timestamp;
        proof_record.reward_distributed = reward_allocation.total_reward;
        proof_record.bump = ctx.bumps.proof_record;

        // New fields in v0.2.0
        proof_record.entropy_score = validated_proof.entropy_score;
        proof_record.is_tensor_aware = if registry.is_tensor_aware() { 1 } else { 0 };
        proof_record.path_distance = params.path_distance;
        proof_record.device_type = miner_account.device_type;

        // 4. Update miner stats
        miner_account.total_proofs += 1;
        miner_account.pending_rewards += reward_allocation.total_reward;
        miner_account.last_proof_slot = clock.slot;
        miner_account.reputation_score += 1;

        // Update tensor fields
        miner_account.entropy_score = validated_proof.entropy_score;
        if registry.is_tensor_aware() {
            miner_account.last_entropy_update = clock.slot;
        }

        // 5. Update global stats
        let cfg = &mut ctx.accounts.config;
        cfg.total_proofs += 1;
        if registry.is_tensor_aware() {
            cfg.network_entropy = cfg
                .network_entropy
                .saturating_add(validated_proof.entropy_score);
        }

        // 6. Emit events
        emit!(ProofSubmitted {
            miner: ctx.accounts.miner.key(),
            challenge_id: params.challenge_id,
            mml_score: params.mml_score,
            slot: clock.slot,
            timestamp: clock.unix_timestamp,
            entropy_score: validated_proof.entropy_score,
            is_tensor_aware: registry.is_tensor_aware(),
        });

        emit!(RewardDistributed {
            miner: ctx.accounts.miner.key(),
            base_reward: reward_allocation.base_reward,
            bonus_reward: reward_allocation.bonus_reward,
            total_reward: reward_allocation.total_reward,
            multiplier: (reward_allocation.multiplier * 1_000_000.0) as u64, // Convert f64 to u64
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Adjust difficulty (admin or time-gated crank).
    ///
    /// Can also adjust entropy weight factor and other tensor parameters.
    pub fn adjust_difficulty(
        ctx: Context<AdjustDifficulty>,
        new_difficulty: u64,
        new_mml_threshold: u64,
        new_path_distance_max: u32,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.difficulty = new_difficulty;
        config.mml_threshold = new_mml_threshold;
        config.path_distance_max = new_path_distance_max;
        Ok(())
    }

    /// Claim accumulated rewards.
    ///
    /// Transfers accumulated pending rewards to the miner's authority.
    /// In production, this would include CPI to SPL token program.
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let miner = &mut ctx.accounts.miner_account;
        let pending = miner.pending_rewards;
        require!(pending > 0, PotOError::NoRewardsToClaim);

        miner.total_rewards += pending;
        miner.pending_rewards = 0;

        // In production: CPI transfer of PTtC/NMTC tokens here.
        // For now, reward accounting is on-chain only.

        let _clock = Clock::get()?;
        msg!("Rewards claimed: {} tokens", pending);

        Ok(())
    }

    /// Update pool configuration (admin only).
    ///
    /// Can enable/disable tensor network features and configure pool parameters.
    pub fn update_pool_config(
        ctx: Context<UpdatePoolConfig>,
        pool_type: u8,
        swap_program_id: Pubkey,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.pool_type = pool_type;
        config.swap_program_id = swap_program_id;
        Ok(())
    }

    /// Enable or update tensor network configuration (admin only).
    ///
    /// Called to enable tensor network features and set entropy parameters.
    /// Parameters:
    /// - enable: 1 to enable, 0 to disable tensor network
    /// - s_max: Maximum entropy (1e6 scale)
    /// - bond_dimension: Quantum bond dimension (typically 2-4)
    /// - max_pool_size: Maximum miners per entanglement pool
    /// - entropy_weight: Entropy weight factor (in 1e6 scale)
    pub fn configure_tensor_network(
        ctx: Context<UpdatePoolConfig>,
        enable: u8,
        s_max: u64,
        bond_dimension: u32,
        max_pool_size: u32,
        entropy_weight: u64,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.tensor_enabled = enable;
        if enable != 0 {
            config.s_max = s_max.max(1_000_000); // Minimum 1e6
            config.bond_dimension = bond_dimension.clamp(2, 8); // Between 2 and 8
            config.max_pool_size = max_pool_size.clamp(2, 512); // Between 2 and 512
            config.entropy_weight_factor = entropy_weight;
        }
        Ok(())
    }

    /// Request a token swap via CPI to tribewarez-swap (extension point).
    pub fn request_swap(
        _ctx: Context<RequestSwap>,
        _from_token_mint: Pubkey,
        _to_token_mint: Pubkey,
        _amount: u64,
    ) -> Result<()> {
        // Extension point: CPI into tribewarez-swap program.
        // Stub: log and succeed for now.
        msg!("Swap request received (CPI to tribewarez-swap pending)");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Compute initial coherence factor for a device type.
///
/// Coherence preservation capabilities (1e6 scale):
/// - Device 0 (CPU): 0.6 - moderate decoherence
/// - Device 1 (GPU): 0.8 - good coherence
/// - Device 2 (ASIC): 1.0 - excellent coherence (baseline)
/// - Device 3 (Mobile): 0.4 - significant decoherence
fn compute_device_coherence(device_type: u8) -> u64 {
    match device_type {
        0 => 600_000,   // CPU: 0.6x
        1 => 800_000,   // GPU: 0.8x
        2 => 1_000_000, // ASIC: 1.0x (baseline)
        3 => 400_000,   // Mobile: 0.4x
        _ => 500_000,   // Unknown: 0.5x (conservative)
    }
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

/// Parameters for initializing the PoT-O configuration.
/// These values are set once during program initialization and can be updated via `update_pool_config`.
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitParams {
    /// Base difficulty threshold for mining proofs
    pub difficulty: u64,
    /// Minimum MML (Merkle Merkle Linkage) score required for valid proofs
    pub mml_threshold: u64,
    /// Maximum neural path distance allowed in tensor network validation
    pub path_distance_max: u32,
    /// Base reward amount distributed per validated proof
    pub reward_per_proof: u64,
    /// Pool type (0 = legacy, 1 = tensor-aware v0.2.0)
    pub pool_type: u8,
    /// Swap program ID for fee collection functionality
    pub swap_program_id: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
/// Parameters for submitting and validating mining proofs.
/// Contains cryptographic commitments and validation data for the proof.
pub struct ProofParams {
    /// Unique challenge identifier for this mining round
    pub challenge_id: [u8; 32],
    /// Solana slot number when the challenge was issued
    pub challenge_slot: u64,
    /// Hash of the tensor computation result
    pub tensor_result_hash: [u8; 32],
    /// Merkle Merkle Linkage score derived from the proof
    pub mml_score: u64,
    /// Signature proving path through the neural network
    pub path_signature: [u8; 32],
    /// Distance traveled in the neural path (must be <= path_distance_max)
    pub path_distance: u32,
    /// Nonce used in the computation
    pub computation_nonce: u64,
    /// Hash of the complete computation (verified against submitted hash)
    pub computation_hash: [u8; 32],
}


/// Global PoT-O configuration account.
/// Stores network-wide settings for proof validation, reward distribution, and tensor network parameters.
/// This is a singleton account (one per program) created by the admin during initialization.
#[account]
pub struct PotOConfig {
    /// Admin/authority account that can update configuration
    pub admin: Pubkey,
    /// Base difficulty threshold for mining proofs
    pub difficulty: u64,
    /// Minimum MML score required for valid proofs
    pub mml_threshold: u64,
    /// Maximum neural path distance allowed
    pub path_distance_max: u32,
    /// Base reward amount per validated proof
    pub reward_per_proof: u64,
    /// Total number of proofs submitted since inception
    pub total_proofs: u64,
    /// Pool type: 0 = legacy v0.1.x, 1 = tensor-aware v0.2.0
    pub pool_type: u8,
    /// Program ID of the swap program for fee collection
    pub swap_program_id: Pubkey,
    /// PDA bump seed for this account
    pub bump: u8,

    // --- v0.2.0 Tensor Network Extensions ---
    // New fields added at end for ABI compatibility
    /// Whether tensor network enhancements are enabled
    pub tensor_enabled: u8,         // 0 = disabled, 1 = enabled
    /// Maximum entropy (1e6 scale)
    pub s_max: u64,                 // Maximum entropy (1e6 scale)
    /// Quantum bond dimension for tensor network calculations
    pub bond_dimension: u32,        // Quantum bond dimension
    /// Maximum miners allowed per entanglement pool
    pub max_pool_size: u32,         // Maximum miners per pool
    /// Weight factor for entropy in reward calculations (1e6 scale)
    pub entropy_weight_factor: u64, // Entropy weight in 1e6 scale
    /// Current network-wide entropy measurement
    pub network_entropy: u64,       // Current network entropy
    /// Total number of active miners in the network
    pub total_miners: u32,          // Number of active miners
    /// Number of active entanglement pools
    pub active_pools: u32,          // Number of entanglement pools
    /// Average coherence score across all devices (1e6 scale)
    pub average_coherence: u64,     // Average device coherence (1e6 scale)

    // Reserved for future expansion (256 bytes total)
    /// Reserved space for future updates without breaking ABI compatibility
    pub reserved: [u8; 200],
}


/// Per-miner account storing mining history and statistics.
/// Created when a miner first registers. Updated each time a proof is submitted.
#[account]
pub struct MinerAccount {
    /// Public key of the miner who owns this account
    pub authority: Pubkey,
    /// Type of hardware used by this miner
    pub device_type: u8,
    /// Total number of valid proofs submitted by this miner
    pub total_proofs: u64,
    /// Total rewards distributed to this miner
    pub total_rewards: u64,
    /// Pending rewards awaiting claim
    pub pending_rewards: u64,
    /// Reputation score based on submission quality (higher is better)
    pub reputation_score: u64,
    /// Solana slot of the last valid proof submission
    pub last_proof_slot: u64,
    /// ID of the tensor pool this miner is associated with (if any)
    pub pool_id: Pubkey,
    /// PDA bump seed for this account
    pub bump: u8,

    // --- v0.2.0 Tensor Network Extensions ---
    // New fields added at end for ABI compatibility
    /// Vertex position in the tensor network graph
    pub vertex_id: u32,           // Position in tensor network graph
    /// Miner's contribution to network entropy (1e6 scale)
    pub entropy_score: u64,       // Miner's entropy contribution (1e6 scale)
    /// Device's ability to preserve quantum coherence (1e6 scale, 0-1000000)
    pub coherence: u64,           // Device coherence preservation (1e6 scale)
    /// Last slot when this miner's entropy metrics were updated
    pub last_entropy_update: u64, // Last slot when entropy was updated
    /// Number of other miners this miner is entangled with
    pub entanglement_count: u32,  // Number of entangled connections
    /// Current pool generation/epoch
    pub pool_generation: u64,     // Generation/epoch of current pool
    /// Probability of unlock based on entropy (1e6 scale, 0-1000000)
    pub unlock_probability: u64,  // P(unlock) calculated from entropy (1e6 scale)

    // Reserved for future expansion (256 bytes total)
    /// Reserved space for future updates without breaking ABI compatibility
    pub reserved: [u8; 192],
}


/// Proof record account storing details of a submitted mining proof.
/// Created each time a miner submits a valid proof.
#[account]
pub struct ProofRecord {
    /// Miner who submitted this proof
    pub miner: Pubkey,
    /// Unique challenge ID for this proof
    pub challenge_id: [u8; 32],
    /// Merkle Merkle Linkage score for this proof
    pub mml_score: u64,
    /// Cryptographic signature proving path through neural network
    pub path_signature: [u8; 32],
    /// Solana slot when the proof was submitted
    pub slot: u64,
    /// Timestamp when the proof was submitted
    pub timestamp: i64,
    /// Reward amount that was distributed for this proof
    pub reward_distributed: u64,
    /// PDA bump seed for this account
    pub bump: u8,

    // --- v0.2.0 Tensor Network Extensions ---
    /// Entropy score calculated from this proof (1e6 scale)
    pub entropy_score: u64,  // Calculated entropy for this proof
    /// Whether this proof was validated using tensor-aware logic
    pub is_tensor_aware: u8, // 0 = standard, 1 = tensor-aware validation
    /// Distance traveled in the neural path
    pub path_distance: u32,  // Neural path distance
    /// Type of device that submitted this proof
    pub device_type: u8,     // Device type that submitted proof
}

// ---------------------------------------------------------------------------
// Contexts
// ---------------------------------------------------------------------------

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 8 + 8 + 4 + 8 + 8 + 1 + 32 + 1 + 1 + 8 + 4 + 4 + 8 + 8 + 4 + 4 + 8 + 200,
        seeds = [b"pot_o_config"],
        bump
    )]
    pub config: Account<'info, PotOConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterMiner<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 1 + 8 + 8 + 8 + 8 + 8 + 32 + 1 + 4 + 8 + 8 + 8 + 4 + 8 + 8 + 192,
        seeds = [b"miner", authority.key().as_ref()],
        bump
    )]
    pub miner_account: Account<'info, MinerAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: ProofParams)]
pub struct SubmitProof<'info> {
    #[account(
        mut,
        seeds = [b"pot_o_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, PotOConfig>,
    /// CHECK: miner pubkey used for PDA derivation; identity is in instruction data
    pub miner: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [b"miner", miner.key().as_ref()],
        bump = miner_account.bump,
    )]
    pub miner_account: Account<'info, MinerAccount>,
    #[account(
        init,
        payer = relayer,
        space = 8 + 32 + 32 + 8 + 32 + 8 + 8 + 8 + 1 + 8 + 1 + 4 + 1,
        seeds = [b"proof", params.challenge_id.as_ref()],
        bump
    )]
    pub proof_record: Account<'info, ProofRecord>,
    #[account(mut)]
    pub relayer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdjustDifficulty<'info> {
    #[account(
        mut,
        seeds = [b"pot_o_config"],
        bump = config.bump,
        has_one = admin,
    )]
    pub config: Account<'info, PotOConfig>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(
        mut,
        seeds = [b"miner", authority.key().as_ref()],
        bump = miner_account.bump,
    )]
    pub miner_account: Account<'info, MinerAccount>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdatePoolConfig<'info> {
    #[account(
        mut,
        seeds = [b"pot_o_config"],
        bump = config.bump,
        has_one = admin,
    )]
    pub config: Account<'info, PotOConfig>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct RequestSwap<'info> {
    #[account(
        seeds = [b"pot_o_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, PotOConfig>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[error_code]
pub enum PotOError {
    #[msg("Challenge has expired (> 256 slots old)")]
    ChallengeExpired,
    #[msg("MML score does not meet the required threshold")]
    MmlThresholdNotMet,
    #[msg("Neural path distance exceeds maximum allowed")]
    PathDistanceTooLarge,
    #[msg("Computation hash does not match expected value")]
    InvalidComputationHash,
    #[msg("No rewards available to claim")]
    NoRewardsToClaim,
}
