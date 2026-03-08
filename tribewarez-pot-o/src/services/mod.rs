/// Proof validation service with support for standard and tensor-aware validators.
pub mod proof_validator;

/// Miner management service with reputation tracking and device-based scaling.
pub mod miner_manager;

/// Reward distribution service with support for simple and tensor-weighted algorithms.
pub mod reward_distributor;

/// Tensor network pool service for entropy calculations and network topology.
pub mod tensor_pool_service;

/// Service registry (DI container) that combines all services.
pub mod registry;

// Re-export main types for convenience
pub use proof_validator::{
    ProofData, ProofValidator, StandardProofValidator, TensorAwareProofValidator, ValidatedProof,
    ValidationError,
};

pub use miner_manager::{MinerInfo, MinerManager, StandardMinerManager, TensorAwareMinerManager};

pub use reward_distributor::{
    RewardAllocation, RewardDistributor, SimpleRewardDistributor, TensorWeightedRewardDistributor,
};

pub use tensor_pool_service::{PoolEdge, PoolVertex, StandardTensorPool, TensorPoolService};

pub use registry::ServiceRegistry;
