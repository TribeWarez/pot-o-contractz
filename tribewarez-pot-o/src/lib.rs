use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hash;

declare_id!("PoToVa1idator11111111111111111111111111111");

/// PoT-O: Proof of Tensor Optimizations on-chain program.
/// Validates mining proofs submitted by the off-chain PoT-O validator,
/// manages miner accounts, distributes rewards, and adjusts difficulty.
#[program]
pub mod tribewarez_pot_o {
    use super::*;

    /// Initialize the PoT-O config. Called once by the admin.
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
        Ok(())
    }

    /// Register a new miner.
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
        Ok(())
    }

    /// Submit a PoT-O proof. Validates on-chain and distributes rewards.
    pub fn submit_proof(ctx: Context<SubmitProof>, params: ProofParams) -> Result<()> {
        let config = &ctx.accounts.config;
        let clock = Clock::get()?;

        // 1. Verify challenge is recent (within 256 slots)
        require!(
            clock.slot.saturating_sub(params.challenge_slot) <= 256,
            PotOError::ChallengeExpired
        );

        // 2. Verify MML score meets threshold
        require!(
            params.mml_score <= config.mml_threshold,
            PotOError::MmlThresholdNotMet
        );

        // 3. Verify path distance
        require!(
            params.path_distance <= config.path_distance_max,
            PotOError::PathDistanceTooLarge
        );

        // 4. Verify computation hash integrity
        let expected_hash = compute_proof_hash(
            &params.challenge_id,
            &params.tensor_result_hash,
            params.mml_score,
            &params.path_signature,
            params.computation_nonce,
        );
        require!(
            expected_hash == params.computation_hash,
            PotOError::InvalidComputationHash
        );

        // 5. Record proof
        let proof_record = &mut ctx.accounts.proof_record;
        proof_record.miner = ctx.accounts.miner.key();
        proof_record.challenge_id = params.challenge_id;
        proof_record.mml_score = params.mml_score;
        proof_record.path_signature = params.path_signature;
        proof_record.slot = clock.slot;
        proof_record.timestamp = clock.unix_timestamp;
        proof_record.reward_distributed = config.reward_per_proof;
        proof_record.bump = ctx.bumps.proof_record;

        // 6. Update miner stats
        let miner = &mut ctx.accounts.miner_account;
        miner.total_proofs += 1;
        miner.pending_rewards += config.reward_per_proof;
        miner.last_proof_slot = clock.slot;
        miner.reputation_score += 1;

        // 7. Update global stats
        let cfg = &mut ctx.accounts.config;
        cfg.total_proofs += 1;

        Ok(())
    }

    /// Adjust difficulty (admin or time-gated crank).
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
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let miner = &mut ctx.accounts.miner_account;
        let pending = miner.pending_rewards;
        require!(pending > 0, PotOError::NoRewardsToClaim);

        miner.total_rewards += pending;
        miner.pending_rewards = 0;

        // In production: CPI transfer of PTtC/NMTC tokens here.
        // For now, reward accounting is on-chain only.

        Ok(())
    }

    /// Update pool configuration (admin only).
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

    /// Request a token swap via CPI to tribewarez-swap (extension point).
    pub fn request_swap(
        ctx: Context<RequestSwap>,
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

fn compute_proof_hash(
    challenge_id: &[u8; 32],
    tensor_result_hash: &[u8; 32],
    mml_score: u64,
    path_signature: &[u8; 32],
    nonce: u64,
) -> [u8; 32] {
    let mut data = Vec::with_capacity(32 + 32 + 8 + 32 + 8);
    data.extend_from_slice(challenge_id);
    data.extend_from_slice(tensor_result_hash);
    data.extend_from_slice(&mml_score.to_le_bytes());
    data.extend_from_slice(path_signature);
    data.extend_from_slice(&nonce.to_le_bytes());
    hash(&data).to_bytes()
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitParams {
    pub difficulty: u64,
    pub mml_threshold: u64,
    pub path_distance_max: u32,
    pub reward_per_proof: u64,
    pub pool_type: u8,
    pub swap_program_id: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ProofParams {
    pub challenge_id: [u8; 32],
    pub challenge_slot: u64,
    pub tensor_result_hash: [u8; 32],
    pub mml_score: u64,
    pub path_signature: [u8; 32],
    pub path_distance: u32,
    pub computation_nonce: u64,
    pub computation_hash: [u8; 32],
}

#[account]
pub struct PotOConfig {
    pub admin: Pubkey,
    pub difficulty: u64,
    pub mml_threshold: u64,
    pub path_distance_max: u32,
    pub reward_per_proof: u64,
    pub total_proofs: u64,
    pub pool_type: u8,
    pub swap_program_id: Pubkey,
    pub bump: u8,
}

#[account]
pub struct MinerAccount {
    pub authority: Pubkey,
    pub device_type: u8,
    pub total_proofs: u64,
    pub total_rewards: u64,
    pub pending_rewards: u64,
    pub reputation_score: u64,
    pub last_proof_slot: u64,
    pub pool_id: Pubkey,
    pub bump: u8,
}

#[account]
pub struct ProofRecord {
    pub miner: Pubkey,
    pub challenge_id: [u8; 32],
    pub mml_score: u64,
    pub path_signature: [u8; 32],
    pub slot: u64,
    pub timestamp: i64,
    pub reward_distributed: u64,
    pub bump: u8,
}

// ---------------------------------------------------------------------------
// Contexts
// ---------------------------------------------------------------------------

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 8 + 8 + 4 + 8 + 8 + 1 + 32 + 1,
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
        space = 8 + 32 + 1 + 8 + 8 + 8 + 8 + 8 + 32 + 1,
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
        space = 8 + 32 + 32 + 8 + 32 + 8 + 8 + 8 + 1,
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
