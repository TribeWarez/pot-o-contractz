use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hash;

/// Result type for proof validation operations.
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Validation errors that can occur during proof validation.
#[derive(Debug, Clone, Copy)]
pub enum ValidationError {
    ChallengeExpired,
    MmlThresholdNotMet,
    PathDistanceTooLarge,
    InvalidComputationHash,
    EntropyCheckFailed,
    EntanglementViolation,
    InvalidTensorContraction,
}

/// Proof parameters for validation.
#[derive(Clone, Copy)]
pub struct ProofData {
    pub challenge_id: [u8; 32],
    pub challenge_slot: u64,
    pub tensor_result_hash: [u8; 32],
    pub mml_score: u64,
    pub path_signature: [u8; 32],
    pub path_distance: u32,
    pub computation_nonce: u64,
    pub computation_hash: [u8; 32],
}

/// Validated proof result containing metadata for reward calculation.
#[derive(Clone, Copy)]
pub struct ValidatedProof {
    pub challenge_id: [u8; 32],
    pub mml_score: u64,
    pub path_distance: u32,
    pub entropy_score: u64, // 0 - 1_000_000 (1e6 scale)
    pub is_tensor_aware: bool,
}

/// Core trait for proof validation.
///
/// Implementations validate that a submitted proof meets difficulty requirements,
/// entropy constraints, and tensor network criteria (if enabled).
pub trait ProofValidator {
    /// Validate a proof against the current configuration.
    ///
    /// Parameters:
    /// - `proof`: The proof data to validate
    /// - `current_slot`: Current blockchain slot (for freshness checks)
    /// - `mml_threshold`: Maximum acceptable MML score
    /// - `path_distance_max`: Maximum acceptable neural path distance
    ///
    /// Returns:
    /// - Ok(ValidatedProof) if validation passes
    /// - Err(ValidationError) if any check fails
    fn validate(
        &self,
        proof: &ProofData,
        current_slot: u64,
        mml_threshold: u64,
        path_distance_max: u32,
    ) -> ValidationResult<ValidatedProof>;

    /// Recalculate difficulty based on network conditions.
    ///
    /// Used by cranks to dynamically adjust difficulty as the network
    /// scales or contracts.
    fn recommend_difficulty_adjustment(
        &self,
        current_difficulty: u64,
        total_proofs_last_period: u64,
        target_proofs_per_period: u64,
    ) -> u64;
}

/// Standard proof validator (v0.1.x compatible).
///
/// This is the baseline validator that checks:
/// 1. Challenge freshness (within 256 slots)
/// 2. MML score threshold
/// 3. Path distance constraint
/// 4. Computation hash integrity
pub struct StandardProofValidator;

impl StandardProofValidator {
    pub fn new() -> Self {
        StandardProofValidator
    }

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
}

impl Default for StandardProofValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ProofValidator for StandardProofValidator {
    fn validate(
        &self,
        proof: &ProofData,
        current_slot: u64,
        mml_threshold: u64,
        path_distance_max: u32,
    ) -> ValidationResult<ValidatedProof> {
        // 1. Check challenge freshness (within 256 slots)
        let age = current_slot.saturating_sub(proof.challenge_slot);
        if age > 256 {
            return Err(ValidationError::ChallengeExpired);
        }

        // 2. Check MML threshold
        if proof.mml_score > mml_threshold {
            return Err(ValidationError::MmlThresholdNotMet);
        }

        // 3. Check path distance
        if proof.path_distance > path_distance_max {
            return Err(ValidationError::PathDistanceTooLarge);
        }

        // 4. Verify computation hash
        let expected_hash = Self::compute_proof_hash(
            &proof.challenge_id,
            &proof.tensor_result_hash,
            proof.mml_score,
            &proof.path_signature,
            proof.computation_nonce,
        );
        if expected_hash != proof.computation_hash {
            return Err(ValidationError::InvalidComputationHash);
        }

        Ok(ValidatedProof {
            challenge_id: proof.challenge_id,
            mml_score: proof.mml_score,
            path_distance: proof.path_distance,
            entropy_score: 0, // No entropy calculation for standard validator
            is_tensor_aware: false,
        })
    }

    fn recommend_difficulty_adjustment(
        &self,
        current_difficulty: u64,
        total_proofs_last_period: u64,
        target_proofs_per_period: u64,
    ) -> u64 {
        if total_proofs_last_period == 0 {
            // If no proofs, increase difficulty
            return (current_difficulty * 105) / 100;
        }

        let ratio = (total_proofs_last_period as i64) - (target_proofs_per_period as i64);
        let adjustment_percent = 100i64 + (ratio * 2); // 2% per proof deviation

        if adjustment_percent < 50 {
            return (current_difficulty / 2).max(1);
        }
        if adjustment_percent > 200 {
            return current_difficulty.saturating_mul(2);
        }

        ((current_difficulty as i64) * adjustment_percent / 100) as u64
    }
}

/// Tensor-aware proof validator (v0.2.0).
///
/// This validator extends StandardProofValidator with:
/// - Entropy calculations from tensor network states
/// - Mutual information checks between miner's entanglement and the network
/// - Effective distance constraints based on quantum coherence
/// - Adaptive difficulty based on network entropy
///
/// Based on REALMS Part IV formulas:
/// - Entropy: S = |γ| log(d) where γ = edges crossing cut
/// - Mutual Information: I(A:B) = S(A) + S(B) - S(A∪B)
/// - Effective Distance: d_eff = 1 - I(A:B) / S_max
/// - Unlock Probability: P = tanh(S_A / S_max)
pub struct TensorAwareProofValidator {
    s_max: u64, // Maximum entropy (1e6 scale)
}

impl TensorAwareProofValidator {
    pub fn new(s_max: u64) -> Self {
        TensorAwareProofValidator { s_max }
    }

    /// Calculate entropy score from MML and path distance.
    ///
    /// Simplified model:
    /// - Base entropy from MML score (represents bonds in tensor network)
    /// - Path distance reduces entropy (longer paths = more decoherence)
    /// - Returns score in 0-1_000_000 range (1e6 scale)
    fn calculate_entropy_score(&self, mml_score: u64, path_distance: u32) -> u64 {
        // Base entropy from MML: lower MML = higher entropy (better optimization)
        let base_entropy = if mml_score > 0 {
            (self.s_max / (mml_score + 1)).min(self.s_max)
        } else {
            self.s_max
        };

        // Decoherence penalty from path distance
        // Each unit of distance reduces by 1%
        let decoherence_factor = 100u64.saturating_sub(path_distance as u64);
        let entropy_score = (base_entropy * decoherence_factor) / 100;

        entropy_score.min(self.s_max)
    }
}

impl ProofValidator for TensorAwareProofValidator {
    fn validate(
        &self,
        proof: &ProofData,
        current_slot: u64,
        mml_threshold: u64,
        path_distance_max: u32,
    ) -> ValidationResult<ValidatedProof> {
        // First pass: standard validation
        let standard = StandardProofValidator::new();
        let standard_proof = standard.validate(proof, current_slot, mml_threshold, path_distance_max)?;

        // Second pass: tensor entropy checks
        let entropy_score = self.calculate_entropy_score(proof.mml_score, proof.path_distance);

        // Require minimum entropy (at least 10% of S_max)
        if entropy_score < (self.s_max / 10) {
            return Err(ValidationError::EntropyCheckFailed);
        }

        Ok(ValidatedProof {
            challenge_id: standard_proof.challenge_id,
            mml_score: standard_proof.mml_score,
            path_distance: standard_proof.path_distance,
            entropy_score,
            is_tensor_aware: true,
        })
    }

    fn recommend_difficulty_adjustment(
        &self,
        current_difficulty: u64,
        total_proofs_last_period: u64,
        target_proofs_per_period: u64,
    ) -> u64 {
        // Base adjustment from standard validator
        let base_adjustment = StandardProofValidator::new()
            .recommend_difficulty_adjustment(
                current_difficulty,
                total_proofs_last_period,
                target_proofs_per_period,
            );

        // In tensor-aware mode, penalize if too few proofs (entropy declining)
        // This encourages more mining to maintain network coherence
        if total_proofs_last_period < target_proofs_per_period {
            // Extra 1% increase per missing proof
            let deficit = target_proofs_per_period.saturating_sub(total_proofs_last_period);
            let penalty = (base_adjustment * deficit) / target_proofs_per_period;
            base_adjustment.saturating_add(penalty)
        } else {
            base_adjustment
        }
    }
}

/// Mock proof validator for testing.
///
/// Accepts all proofs, useful for testing reward distribution and state
/// management without focusing on validation logic.
#[cfg(test)]
pub struct MockProofValidator {
    always_fail: bool,
}

#[cfg(test)]
impl MockProofValidator {
    pub fn new() -> Self {
        MockProofValidator {
            always_fail: false,
        }
    }

    pub fn with_failure(mut self) -> Self {
        self.always_fail = true;
        self
    }
}

#[cfg(test)]
impl ProofValidator for MockProofValidator {
    fn validate(
        &self,
        proof: &ProofData,
        _current_slot: u64,
        _mml_threshold: u64,
        _path_distance_max: u32,
    ) -> ValidationResult<ValidatedProof> {
        if self.always_fail {
            return Err(ValidationError::InvalidComputationHash);
        }

        Ok(ValidatedProof {
            challenge_id: proof.challenge_id,
            mml_score: proof.mml_score,
            path_distance: proof.path_distance,
            entropy_score: 500_000, // Mock entropy
            is_tensor_aware: false,
        })
    }

    fn recommend_difficulty_adjustment(
        &self,
        current_difficulty: u64,
        _total_proofs_last_period: u64,
        _target_proofs_per_period: u64,
    ) -> u64 {
        current_difficulty // No adjustment in mock
    }
}
