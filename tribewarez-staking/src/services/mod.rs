/// Staking reward calculator service (time-based and tensor-aware variants).
pub mod staking_calculator;

/// Entanglement pool service for tensor network stake coupling.
pub mod entanglement_service;

// Re-export main types for convenience
pub use staking_calculator::{
    StakingCalculator, StakingError, StakingResult,
    SimpleStakingCalculator, TensorAwareStakingCalculator,
};

pub use entanglement_service::{
    EntanglementService, PoolEntanglement,
    SimpleEntanglementService, TensorEntanglementService,
};
