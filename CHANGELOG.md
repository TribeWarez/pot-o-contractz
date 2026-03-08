# Changelog

All notable changes to pot-o-contractz are documented in this file.

## [0.2.0] - 2026-03-08

### Major Features

#### Dependency Injection (DI) Service Architecture
- **Implemented across all programs** (pot-o, staking, vault, swap)
- **ServiceRegistry pattern**: Centralized DI container for instantiating services
- **Multiple implementations per trait**: Legacy (v0.1.x compatible) and TensorAware (v0.2.0 quantum-inspired)
- **Benefits**: Seamless feature toggles, testability, maintainability without code duplication

#### Tensor Network Model Integration (REALMS Part IV)
- **Quantum-inspired calculations** based on entanglement entropy and mutual information
- **Fixed-point arithmetic at 1e6 scale** ensuring 6 decimal precision
- **Core formulas**:
  - **Entropy**: S = |γ| log(d) where γ = edges crossing cut, d = bond dimension
  - **Mutual Information**: I(A:B) = S(A) + S(B) - S(A∪B)
  - **Effective Distance**: d_eff = 1 - I(A:B) / S_max
  - **Coherence Probability**: P(unlock) = tanh(S_A / S_max)

#### Device Coherence Factors
- **ASIC (type 2)**: 1.0x baseline multiplier
- **GPU (type 1)**: 0.8x multiplier
- **CPU (type 0)**: 0.6x multiplier
- **Mobile (type 3)**: 0.4x multiplier
- Applied to rewards, unlock probability, and fee discounts

### Program Updates

#### tribewarez-pot-o v0.2.0
- **New Services**:
  - `ProofValidator`: Abstract proof validation (Standard + TensorAware implementations)
  - `MinerManager`: Miner lifecycle management with entropy tracking
  - `RewardDistributor`: Token distribution (SimpleRewardDistributor + TensorWeightedRewardDistributor)
  - `TensorPoolService`: Entropy, mutual information, and effective distance calculations
  - `ServiceRegistry`: DI container and configuration management

- **New Events**:
  - `MinerRegistered`: Emitted when miner joins pool
  - `ProofSubmitted`: Emitted when miner submits valid proof
  - `RewardDistributed`: Emitted when rewards distributed to miners
  - `EntropyStateUpdated`: Emitted when tensor state changes

- **State Extensions** (ABI compatible, appended to existing structs):
  - `PotOConfig`: Added 9 tensor fields (entropy_weight, mutual_info_scale, etc.)
  - `MinerAccount`: Added 7 tensor fields (individual entropy, coherence, etc.)
  - `ProofRecord`: Added 4 fields for tensor tracking

- **Features**:
  - Tensor-weighted reward distribution (0-20% variance based on pool entropy)
  - Coherence-based proof acceptance (variable difficulty per device type)
  - Entropy-aware difficulty adjustment

#### tribewarez-staking v0.2.0
- **New Services**:
  - `StakingCalculator`: Reward calculations (SimpleStakingCalculator + TensorAwareStakingCalculator)
  - `EntanglementService`: Pool coupling and mutual information tracking
  - ServiceRegistry DI container

- **New Events**:
  - `Staked`: Emitted on stake deposit
  - `Unstaked`: Emitted on stake withdrawal
  - `RewardsClaimed`: Emitted when staker claims rewards
  - `StakeEntangled`: Emitted when stake enters entangled state

- **Features**:
  - Time-based rewards with unlock probability: P(unlock) = tanh(S_A / S_max)
  - Coherence bonuses (0-10% APY bonus at max coherence)
  - Pool efficiency multipliers (0-20% based on total pool entropy)
  - Dynamic unlock periods based on entanglement entropy

- **State Extensions**:
  - `StakingPool`: Added 6 tensor fields
  - `StakeAccount`: Added 6 tensor fields

#### tribewarez-vault v0.2.0
- **New Services**:
  - `VaultSecurityProvider`: Locktime management (SimpleVaultSecurity + TensorVaultSecurity)
  - ServiceRegistry configuration

- **New Events**:
  - `VaultCreated`: Emitted on vault initialization
  - `Deposited`: Emitted on deposit
  - `Withdrawn`: Emitted on withdrawal
  - `VaultUnlocked`: Emitted when locktime expires or conditions met

- **Features**:
  - Dynamic locktime reduction (0-100% reduction based on entropy)
  - Dynamic early withdrawal fees (0-50% reduction at max entropy)
  - Escrow functionality for time-locked funds
  - Coherence-based locktime adjustments

#### tribewarez-swap v0.2.0
- **New Services**:
  - `SwapCalculator`: AMM calculations (SimpleSwapCalculator + TensorSwapCalculator)
  - ServiceRegistry configuration

- **New Events**:
  - `PoolInitialized`: Emitted on pool creation
  - `LiquidityAdded`: Emitted when LP adds liquidity
  - `LiquidityRemoved`: Emitted when LP removes liquidity
  - `SwapExecuted`: Emitted when swap completes

- **Features**:
  - Constant product AMM (x * y = k formula)
  - Standard fee structure (0.30% swap fee, 0.05% protocol fee)
  - Coherence-based fee discounts (0-50% reduction at max coherence)
  - Price impact calculation and slippage validation
  - Dynamic fee adjustments based on miner/staker coherence

### Testing & Validation

#### Unit Tests (120+ tests, 2,500+ lines)
- **Reward distribution tests** (15 tests): SimpleRewardDistributor vs TensorWeightedRewardDistributor
- **Tensor formula tests** (30 tests): All REALMS Part IV equations verified
- **Staking calculator tests** (25 tests): Time-based, entropy-aware, coherence bonuses
- **Vault security tests** (20 tests): Lock/unlock, early withdrawal fees, dynamic reductions
- **Swap calculator tests** (30+ tests): AMM formula, fee structures, price impact

#### Integration Tests (10+ scenarios, 350 lines)
- Full cycle testing: Miner → Staker → Vault → Swap
- Cross-contract event propagation
- State consistency validation
- Tensor entropy propagation through system

#### Backward Compatibility Tests (15+ validations, 400 lines)
- Instruction signatures unchanged ✓
- State layout extends without breaking ✓
- Legacy mode produces identical v0.1.x behavior ✓
- v0.1.x and v0.2.0 clients compatible ✓

### Breaking Changes

**None!** - Full ABI backward compatibility maintained:
- Instruction signatures identical to v0.1.x
- New tensor fields appended to state structs
- Legacy mode produces v0.1.x calculation results
- Zero migration required for existing accounts

### Documentation

- **PHASE6_TESTING_SUMMARY.md**: Comprehensive testing documentation
- **Service architecture pattern documentation**: DI design patterns
- **Tensor formula specifications**: REALMS Part IV implementation details
- **Migration guide**: Upgrade path from v0.1.x to v0.2.0

### Dependencies

- **anchor-lang**: 0.29.x
- **anchor-spl**: 0.29.x
- **solana-program**: 1.18.x
- **pot-o-core**: v0.2.0 (new, provides tensor network types and calculations)

### Known Limitations

1. Fixed-point math at 1e6 scale: Maximum precision is 6 decimal places
2. Tensor calculations assume 2-partition bond dimension (d=2) for simplicity
3. TensorAware mode requires additional compute for entropy/MI calculations (~15-20% overhead)
4. Device coherence factors are fixed and cannot be updated after program deployment

### Migration Path from v0.1.x

1. **Deploy new v0.2.0 programs** alongside existing v0.1.x programs
2. **No account migration needed** - v0.2.0 can read v0.1.x accounts directly
3. **Configure ServiceRegistry** to use legacy implementations for drop-in replacement
4. **Gradually enable tensor features** as needed:
   - Start with legacy mode (SimpleRewardDistributor, SimpleStakingCalculator, etc.)
   - Enable TensorAware implementations per instruction/feature
   - Monitor for expected behavior changes

### Future Roadmap (Post v0.2.0)

- [ ] Variable device coherence factor updates
- [ ] Higher-order tensor network models (3+ partitions)
- [ ] Quantum circuit simulation on Solana
- [ ] Dynamic bond dimension adjustment
- [ ] Cross-program tensor entanglement
- [ ] Off-chain tensor computation proofs

### Contributors

- Tensor network implementation: pot-o development team
- REALMS Part IV specifications: Research team
- Testing and validation: QA team

---

## [0.1.x] (Legacy)

Previous v0.1.x releases included:
- Basic proof-of-work mining with difficulty adjustment
- Simple reward distribution (linear per share)
- Staking with fixed APY
- Vault with static locktimes
- Constant product AMM without coherence bonuses
- No service abstraction (monolithic contract logic)

See git history for v0.1.x changelog and releases.
