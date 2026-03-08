use super::{
    MinerManager, ProofValidator, RewardDistributor, SimpleRewardDistributor, StandardMinerManager,
    StandardProofValidator, StandardTensorPool, TensorAwareMinerManager, TensorAwareProofValidator,
    TensorPoolService, TensorWeightedRewardDistributor,
};

/// Service registry that combines all DI services for proof validation and reward distribution.
///
/// This registry can be configured in two modes:
/// 1. **Legacy Mode** (v0.1.x): Simple proof validation and fixed rewards
/// 2. **Tensor-Aware Mode** (v0.2.0): Full tensor network validation and entropy-weighted rewards
///
/// The registry pattern decouples business logic from instruction handlers,
/// making code testable and maintainable.
pub enum ServiceRegistry {
    /// Legacy configuration: simple, stateless validators
    Legacy {
        proof_validator: StandardProofValidator,
        miner_manager: StandardMinerManager,
        reward_distributor: SimpleRewardDistributor,
    },
    /// Tensor-aware configuration: full v0.2.0 feature set
    TensorAware {
        proof_validator: TensorAwareProofValidator,
        miner_manager: TensorAwareMinerManager,
        reward_distributor: TensorWeightedRewardDistributor,
        tensor_pool: Box<dyn TensorPoolService>,
    },
}

impl ServiceRegistry {
    /// Create a legacy registry (v0.1.x compatible).
    ///
    /// # Example
    /// ```ignore
    /// let registry = ServiceRegistry::new_legacy();
    /// ```
    pub fn new_legacy() -> Self {
        ServiceRegistry::Legacy {
            proof_validator: StandardProofValidator::new(),
            miner_manager: StandardMinerManager::new(),
            reward_distributor: SimpleRewardDistributor::new(),
        }
    }

    /// Create a tensor-aware registry (v0.2.0).
    ///
    /// # Parameters
    /// - `s_max`: Maximum entropy (1e6 scale, typically 1_000_000)
    /// - `bond_dimension`: Quantum bond dimension (typically 2 or 4)
    /// - `entropy_weight`: Entropy contribution weight for rewards (e.g., 0.5 = 50% bonus at max entropy)
    ///
    /// # Example
    /// ```ignore
    /// let registry = ServiceRegistry::new_tensor_aware(
    ///     1_000_000,  // S_max
    ///     2,          // bond_dimension
    ///     0.5,        // entropy_weight
    ///     128,        // max_pool_size
    /// );
    /// ```
    pub fn new_tensor_aware(
        s_max: u64,
        bond_dimension: u32,
        entropy_weight: f64,
        max_pool_size: u32,
    ) -> Self {
        ServiceRegistry::TensorAware {
            proof_validator: TensorAwareProofValidator::new(s_max),
            miner_manager: TensorAwareMinerManager::new(max_pool_size, entropy_weight),
            reward_distributor: TensorWeightedRewardDistributor::new(s_max, entropy_weight),
            tensor_pool: Box::new(StandardTensorPool::new(s_max, bond_dimension)),
        }
    }

    /// Get a reference to the proof validator.
    pub fn proof_validator(&self) -> &dyn ProofValidator {
        match self {
            ServiceRegistry::Legacy {
                proof_validator, ..
            } => proof_validator,
            ServiceRegistry::TensorAware {
                proof_validator, ..
            } => proof_validator,
        }
    }

    /// Get a reference to the miner manager.
    pub fn miner_manager(&self) -> &dyn MinerManager {
        match self {
            ServiceRegistry::Legacy { miner_manager, .. } => miner_manager,
            ServiceRegistry::TensorAware { miner_manager, .. } => miner_manager,
        }
    }

    /// Get a reference to the reward distributor.
    pub fn reward_distributor(&self) -> &dyn RewardDistributor {
        match self {
            ServiceRegistry::Legacy {
                reward_distributor, ..
            } => reward_distributor,
            ServiceRegistry::TensorAware {
                reward_distributor, ..
            } => reward_distributor,
        }
    }

    /// Get a mutable reference to the tensor pool service (if available).
    pub fn tensor_pool_mut(&mut self) -> Option<&mut dyn TensorPoolService> {
        match self {
            ServiceRegistry::Legacy { .. } => None,
            ServiceRegistry::TensorAware { tensor_pool, .. } => Some(tensor_pool.as_mut()),
        }
    }

    /// Check if this registry is in tensor-aware mode.
    pub fn is_tensor_aware(&self) -> bool {
        matches!(self, ServiceRegistry::TensorAware { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_registry() {
        let mut registry = ServiceRegistry::new_legacy();
        assert!(!registry.is_tensor_aware());
        assert!(registry.tensor_pool_mut().is_none());
    }

    #[test]
    fn test_tensor_aware_registry() {
        let mut registry = ServiceRegistry::new_tensor_aware(1_000_000, 2, 0.5, 128);
        assert!(registry.is_tensor_aware());
        assert!(registry.tensor_pool_mut().is_some());
    }

    #[test]
    fn test_proof_validator_access() {
        let registry = ServiceRegistry::new_legacy();
        let _validator = registry.proof_validator();
        // Can use validator trait methods
    }

    #[test]
    fn test_reward_distributor_access() {
        let registry = ServiceRegistry::new_legacy();
        let distributor = registry.reward_distributor();
        let allocation = distributor.calculate_reward(1000, 0, Default::default(), 0);
        assert_eq!(allocation.total_reward, 1000);
    }
}
