# Service API Reference

Documentation of the trait-based service architecture used across tribewarez programs for dependency injection, testing, and integration patterns.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Core Service Traits](#core-service-traits)
3. [Service Registry Pattern](#service-registry-pattern)
4. [Custom Implementation Examples](#custom-implementation-examples)
5. [Error Handling](#error-handling)
6. [Integration Patterns](#integration-patterns)
7. [Testing and Mocks](#testing-and-mocks)
8. [Service Lifecycle](#service-lifecycle)
9. [Performance Considerations](#performance-considerations)

---

## Architecture Overview

The tribewarez ecosystem uses **trait-based dependency injection** to decouple program logic from implementation details. This pattern enables:

- **Testability**: Mock implementations for unit tests
- **Flexibility**: Swap implementations without recompiling
- **Reusability**: Share service logic across programs
- **Extensibility**: Add new services without modifying existing code

### Service Layers

```
┌─────────────────────────────────────────┐
│         Program Instructions             │
│         (PoT-O, Staking, Vault, etc.)   │
└──────────────────┬──────────────────────┘
                   │
        ┌──────────┴──────────┐
        │                     │
        ▼                     ▼
┌──────────────────┐  ┌──────────────────┐
│  Service Traits  │  │   Service Trait  │
│  (Abstract Iface)│  │  Implementations │
└──────────────────┘  └──────────────────┘
        │                     │
        └──────────┬──────────┘
                   │
        ┌──────────▼──────────┐
        │   On-Chain Solana   │
        │   Token Program,    │
        │   Clock Sysvar, etc │
        └─────────────────────┘
```

---

## Core Service Traits

### 1. ChainBridge Service

Bridges on-chain Solana state with off-chain computation.

```rust
pub trait ChainBridge {
    /// Get current slot number
    fn get_slot(&self) -> Result<u64>;
    
    /// Get current Unix timestamp
    fn get_unix_timestamp(&self) -> Result<i64>;
    
    /// Verify account ownership
    fn verify_owner(&self, account: &Pubkey, owner: &Pubkey) -> Result<bool>;
    
    /// Get account balance in lamports
    fn get_lamport_balance(&self, account: &Pubkey) -> Result<u64>;
    
    /// Check if account exists on-chain
    fn account_exists(&self, account: &Pubkey) -> Result<bool>;
}
```

**Usage**: Clock-dependent logic, account validation, state checking

**Implementations**:
- **SolanaChainBridge**: Real Solana Clock sysvar
- **MockChainBridge**: Testnet mock with configurable time

---

### 2. StakingService

Manages staking pool operations and reward calculations.

```rust
pub trait StakingService {
    /// Stake tokens in a pool
    fn stake(
        &mut self,
        pool: &Pubkey,
        staker: &Pubkey,
        amount: u64,
    ) -> Result<()>;
    
    /// Unstake tokens from a pool
    fn unstake(
        &mut self,
        pool: &Pubkey,
        staker: &Pubkey,
        amount: u64,
    ) -> Result<()>;
    
    /// Calculate pending rewards
    fn calculate_rewards(
        &self,
        pool: &Pubkey,
        staker: &Pubkey,
    ) -> Result<u64>;
    
    /// Claim accumulated rewards
    fn claim_rewards(
        &mut self,
        pool: &Pubkey,
        staker: &Pubkey,
    ) -> Result<u64>;
    
    /// Get pool configuration
    fn get_pool_config(&self, pool: &Pubkey) -> Result<PoolConfig>;
}
```

**Usage**: Staking operations, reward calculations, pool management

**Configurations**:
- Multiple reward token types (configurable)
- Custom reward schedules
- Emission rates per block

---

### 3. SwapCalculator

Computes AMM swap outputs and pricing.

```rust
pub trait SwapCalculator {
    /// Calculate output for input amount
    fn calculate_output(
        &self,
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
    ) -> Result<u64>;
    
    /// Calculate price impact (basis points)
    fn calculate_price_impact(
        &self,
        amount_in: u64,
        reserve_in: u64,
    ) -> Result<u64>;
    
    /// Calculate fees for transaction
    fn calculate_fees(
        &self,
        amount: u64,
        fee_type: FeeType,
    ) -> Result<u64>;
    
    /// Get liquidity pool state
    fn get_pool_state(&self, pool: &Pubkey) -> Result<PoolState>;
}
```

**Fee Types**:
- `Swap`: Base swap fee (0.30%)
- `Protocol`: Protocol fee allocation (0.05%)
- `LP`: Liquidity provider fee (0.25%)

**Formulas**:
- Constant product: `x * y = k`
- Output: `out = (in * reserve_out * (10000 - fee)) / (reserve_in * 10000 + in * (10000 - fee))`

---

### 4. VaultService

Manages vault and escrow operations.

```rust
pub trait VaultService {
    /// Create a time-locked vault
    fn create_vault(
        &mut self,
        owner: &Pubkey,
        lock_until: i64,
    ) -> Result<Pubkey>;
    
    /// Deposit into vault
    fn deposit(
        &mut self,
        vault: &Pubkey,
        amount: u64,
    ) -> Result<()>;
    
    /// Withdraw from vault
    fn withdraw(
        &mut self,
        vault: &Pubkey,
        amount: u64,
    ) -> Result<()>;
    
    /// Create escrow agreement
    fn create_escrow(
        &mut self,
        depositor: &Pubkey,
        beneficiary: &Pubkey,
        amount: u64,
        release_time: i64,
    ) -> Result<Pubkey>;
    
    /// Check if vault is locked
    fn is_locked(&self, vault: &Pubkey) -> Result<bool>;
}
```

**Temporal Logic**:
- Time-based lock enforcement
- Escrow release conditions
- Lock extension (no reduction)

---

### 5. MiningService

Manages mining operations and proof validation.

```rust
pub trait MiningService {
    /// Register a miner
    fn register_miner(&mut self, authority: &Pubkey) -> Result<Pubkey>;
    
    /// Submit a proof
    fn submit_proof(
        &mut self,
        miner: &Pubkey,
        computation_hash: [u8; 32],
        proof_path: Vec<[u8; 32]>,
    ) -> Result<()>;
    
    /// Award mining rewards
    fn award_reward(
        &mut self,
        miner: &Pubkey,
        amount: u64,
    ) -> Result<()>;
    
    /// Get miner statistics
    fn get_miner_stats(&self, miner: &Pubkey) -> Result<MinerStats>;
    
    /// Adjust difficulty
    fn adjust_difficulty(&mut self, new_difficulty: u64) -> Result<()>;
}
```

**Difficulty Management**:
- Target block time: 150 slots (~2.5 minutes)
- Adjustment interval: 2016 blocks
- Max change: ±25% per adjustment

---

## Service Registry Pattern

The service registry provides centralized access to all services:

```rust
pub struct ServiceRegistry {
    chain_bridge: Arc<dyn ChainBridge>,
    staking_service: Arc<dyn StakingService>,
    swap_calculator: Arc<dyn SwapCalculator>,
    vault_service: Arc<dyn VaultService>,
    mining_service: Arc<dyn MiningService>,
}

impl ServiceRegistry {
    /// Create with all services
    pub fn new(
        chain_bridge: Arc<dyn ChainBridge>,
        staking_service: Arc<dyn StakingService>,
        swap_calculator: Arc<dyn SwapCalculator>,
        vault_service: Arc<dyn VaultService>,
        mining_service: Arc<dyn MiningService>,
    ) -> Self {
        Self {
            chain_bridge,
            staking_service,
            swap_calculator,
            vault_service,
            mining_service,
        }
    }
    
    /// Get specific service
    pub fn chain_bridge(&self) -> &dyn ChainBridge {
        self.chain_bridge.as_ref()
    }
    
    /// Get staking service
    pub fn staking(&self) -> &dyn StakingService {
        self.staking_service.as_ref()
    }
    
    // ... other accessors
}
```

### Default Implementation

For on-chain programs, the default implementations use Solana's instruction context:

```rust
pub struct SolanaServiceRegistry;

impl SolanaServiceRegistry {
    pub fn default() -> ServiceRegistry {
        ServiceRegistry::new(
            Arc::new(SolanaChainBridge),
            Arc::new(SolanaStakingService),
            Arc::new(SolanaSwapCalculator),
            Arc::new(SolanaVaultService),
            Arc::new(SolanaMinedService),
        )
    }
}
```

---

## Custom Implementation Examples

### Example 1: Tensor-Aware Fee Calculator

Custom SwapCalculator with reduced fees for tensor network participants:

```rust
pub struct TensorAwareSwapCalculator {
    base_fee: u64,
    tensor_discount: f64,
}

impl SwapCalculator for TensorAwareSwapCalculator {
    fn calculate_fees(
        &self,
        amount: u64,
        fee_type: FeeType,
    ) -> Result<u64> {
        let base_fee = match fee_type {
            FeeType::Swap => 30,      // 0.30%
            FeeType::Protocol => 5,   // 0.05%
            FeeType::LP => 25,        // 0.25%
        };
        
        // Apply tensor discount
        let discounted_fee = (base_fee as f64 * (1.0 - self.tensor_discount)) as u64;
        
        Ok((amount as u128)
            .checked_mul(discounted_fee as u128)?
            .checked_div(10000)?
            as u64)
    }
}
```

### Example 2: Multi-Token Staking Service

Custom StakingService supporting multiple reward tokens:

```rust
pub struct MultiTokenStakingService {
    reward_tokens: HashMap<Pubkey, RewardConfig>,
}

impl StakingService for MultiTokenStakingService {
    fn calculate_rewards(
        &self,
        pool: &Pubkey,
        staker: &Pubkey,
    ) -> Result<u64> {
        let stake_amount = self.get_stake_amount(pool, staker)?;
        let mut total_rewards = 0u64;
        
        // Calculate rewards for each reward token
        for (token_mint, config) in &self.reward_tokens {
            let token_rewards = stake_amount
                .checked_mul(config.emission_per_block)?
                .checked_div(config.pool_size)?;
            total_rewards = total_rewards.checked_add(token_rewards)?;
        }
        
        Ok(total_rewards)
    }
}
```

### Example 3: Dynamic Vault Lock Service

Custom VaultService with network-dependent lock reductions:

```rust
pub struct DynamicVaultService {
    chain_bridge: Arc<dyn ChainBridge>,
    base_lock_reduction: u64,
}

impl VaultService for DynamicVaultService {
    fn is_locked(&self, vault: &Pubkey) -> Result<bool> {
        let vault_data = self.get_vault(vault)?;
        let current_time = self.chain_bridge.get_unix_timestamp()?;
        
        // Apply network-dependent reduction
        let adjusted_lock_time = vault_data.lock_until
            .saturating_sub(self.base_lock_reduction);
        
        Ok(current_time < adjusted_lock_time)
    }
}
```

---

## Error Handling

All service traits use `Result<T>` for error handling:

```rust
pub type Result<T> = std::result::Result<T, TribeError>;

#[error_code]
pub enum TribeError {
    // ChainBridge errors
    #[msg("Account does not exist")]
    AccountNotFound,
    
    #[msg("Invalid account owner")]
    InvalidOwner,
    
    // StakingService errors
    #[msg("Pool not initialized")]
    PoolNotInitialized,
    
    #[msg("Insufficient stake balance")]
    InsufficientStake,
    
    // SwapCalculator errors
    #[msg("Slippage exceeded")]
    SlippageExceeded,
    
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    
    // VaultService errors
    #[msg("Vault is locked")]
    VaultLocked,
    
    #[msg("Cannot reduce lock time")]
    CannotReduceLock,
    
    // MiningService errors
    #[msg("Miner not registered")]
    MinerNotRegistered,
    
    #[msg("Invalid proof")]
    InvalidProof,
    
    // Generic errors
    #[msg("Math overflow")]
    MathOverflow,
    
    #[msg("Unauthorized")]
    Unauthorized,
}
```

### Error Propagation

Services propagate errors up to instruction handlers:

```rust
pub fn execute_swap(
    ctx: Context<Swap>,
    registry: &ServiceRegistry,
) -> Result<()> {
    let swap_service = registry.swap_calculator();
    
    // Error propagates automatically via ? operator
    let output = swap_service.calculate_output(
        amount_in,
        reserve_in,
        reserve_out,
    )?;
    
    if output < min_amount_out {
        return Err(TribeError::SlippageExceeded.into());
    }
    
    Ok(())
}
```

---

## Integration Patterns

### Pattern 1: Service Injection in Instructions

```rust
#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    // ... other accounts
}

pub fn initialize_pool(
    ctx: Context<InitializePool>,
    registry: &ServiceRegistry,
) -> Result<()> {
    let swap_calc = registry.swap_calculator();
    
    // Use service to validate pool parameters
    let pool_state = swap_calc.get_pool_state(&ctx.accounts.pool.key())?;
    
    // ... rest of initialization
    Ok(())
}
```

### Pattern 2: Service-Driven State Updates

```rust
pub fn claim_rewards(
    ctx: Context<ClaimRewards>,
    registry: &ServiceRegistry,
) -> Result<()> {
    let staking = registry.staking();
    
    // Service calculates and claims rewards
    let reward_amount = staking.claim_rewards(
        &ctx.accounts.pool.key(),
        &ctx.accounts.staker.key(),
    )?;
    
    // Transfer rewards to user
    transfer_tokens(&ctx, reward_amount)?;
    
    Ok(())
}
```

### Pattern 3: Cross-Service Coordination

```rust
pub fn swap_and_stake(
    ctx: Context<SwapAndStake>,
    registry: &ServiceRegistry,
    swap_amount: u64,
) -> Result<()> {
    // Service 1: Calculate swap output
    let swap_calc = registry.swap_calculator();
    let token_out = swap_calc.calculate_output(
        swap_amount,
        reserve_a,
        reserve_b,
    )?;
    
    // Service 2: Stake the swapped tokens
    let staking = registry.staking();
    staking.stake(
        &ctx.accounts.pool.key(),
        &ctx.accounts.staker.key(),
        token_out,
    )?;
    
    Ok(())
}
```

---

## Testing and Mocks

### Mock Service Implementation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    pub struct MockChainBridge {
        current_time: i64,
        current_slot: u64,
    }
    
    impl ChainBridge for MockChainBridge {
        fn get_unix_timestamp(&self) -> Result<i64> {
            Ok(self.current_time)
        }
        
        fn get_slot(&self) -> Result<u64> {
            Ok(self.current_slot)
        }
        
        // ... other methods
    }
    
    #[test]
    fn test_vault_lock_enforcement() {
        let mut mock_bridge = MockChainBridge {
            current_time: 1000,
            current_slot: 100,
        };
        
        // Create vault with lock until 2000
        let vault = VaultService::create_vault(
            &mut service,
            &Pubkey::new_unique(),
            2000,
        ).unwrap();
        
        // Should be locked
        assert!(service.is_locked(&vault).unwrap());
        
        // Advance time
        mock_bridge.current_time = 2001;
        
        // Should no longer be locked
        assert!(!service.is_locked(&vault).unwrap());
    }
}
```

### Mock Registry for Testing

```rust
pub fn create_test_registry() -> ServiceRegistry {
    ServiceRegistry::new(
        Arc::new(MockChainBridge::default()),
        Arc::new(MockStakingService::default()),
        Arc::new(MockSwapCalculator::default()),
        Arc::new(MockVaultService::default()),
        Arc::new(MockMiningService::default()),
    )
}
```

---

## Service Lifecycle

### Initialization

1. **Program Init**: Create service registry with default implementations
2. **Instruction Entry**: Receive context + registry
3. **Service Setup**: Configure dynamic parameters if needed

### Execution

1. **Service Call**: Request operation from trait
2. **Validation**: Service validates inputs
3. **State Update**: Service modifies state if needed
4. **Return**: Result propagated back to instruction

### Cleanup

1. **Event Emission**: Log important state changes
2. **Account Finalization**: Persist final state
3. **Error Handling**: Revert entire transaction if any step fails

---

## Performance Considerations

### Trait Object Overhead

Trait objects (`dyn Trait`) have minimal overhead:
- Virtual function call: ~0.5-1 CU (compute unit)
- Per instruction: negligible impact

### Service Caching

For frequently-accessed services, cache the reference:

```rust
pub fn execute_swaps(
    ctx: Context<MultiSwap>,
    registry: &ServiceRegistry,
) -> Result<()> {
    let swap_calc = registry.swap_calculator();  // Cache once
    
    // Use same reference for multiple calls
    for swap in swaps {
        let output = swap_calc.calculate_output(
            swap.amount_in,
            swap.reserve_in,
            swap.reserve_out,
        )?;
        // ...
    }
    
    Ok(())
}
```

### Optimization Tips

1. **Minimize Service Calls**: Group related operations
2. **Batch Operations**: Process multiple items in single call
3. **Lazy Evaluation**: Calculate only needed values
4. **Cache Results**: Store expensive calculations

---

## Related Documentation

- [tribewarez-pot-o README](tribewarez-pot-o/README.md) - Mining service implementation
- [tribewarez-staking README](tribewarez-staking/README.md) - Staking service implementation
- [tribewarez-vault README](tribewarez-vault/README.md) - Vault service implementation
- [tribewarez-swap README](tribewarez-swap/README.md) - Swap calculator implementation
- [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md) - Deployment instructions