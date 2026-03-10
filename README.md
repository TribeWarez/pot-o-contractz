# Tribewarez DeFi Smart Contracts

[![CI](https://img.shields.io/github/actions/workflow/status/TribeWarez/pot-o-contractz/ci.yml?branch=main&label=CI)](https://github.com/TribeWarez/pot-o-contractz/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/actions/workflow/status/TribeWarez/pot-o-contractz/release.yml?label=Release)](https://github.com/TribeWarez/pot-o-contractz/actions/workflows/release.yml)
[![Auto Tag](https://img.shields.io/github/actions/workflow/status/TribeWarez/pot-o-contractz/tag.yml?branch=main&label=Auto+Tag)](https://github.com/TribeWarez/pot-o-contractz/actions/workflows/tag.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Solana programs (smart contracts) for the Tribewarez DeFi platform supporting PTtC (Pumped TRIBE-Test Coin) operations.

**v0.2.x**: Dependency Injection architecture with Tensor Network integration for quantum-inspired reward calculations.

## Programs Overview

### 0. PoT-O Program (`tribewarez-pot-o`)
- **Purpose**: On-chain validation of Proof of Tensor Optimizations mining proofs
- **Features**:
  - Config and miner registration
  - Submit and validate PoT-O proofs (MML threshold, path distance, computation hash)
  - Reward distribution and difficulty adjustment
  - Claim rewards (TW-RPC-001 aligned)

### 1. Staking Program (`tribewarez-staking`)
- **Purpose**: Stake PTtC tokens to earn rewards over time
- **Features**:
  - Flexible and time-locked staking pools
  - Configurable reward rates (APY)
  - Compound rewards
  - Emergency unstake (forfeit rewards)
  - Admin pool management

### 2. Vault Program (`tribewarez-vault`)
- **Purpose**: Secure token storage and escrow functionality
- **Features**:
  - Personal vaults for users
  - Time-locked savings accounts
  - Two-party escrow with release conditions
  - Deposit/withdrawal with audit trail

### 3. Swap Program (`tribewarez-swap`)
- **Purpose**: Automated Market Maker (AMM) for token swaps
- **Features**:
  - Constant product formula (x * y = k)
  - Liquidity provision with LP tokens
  - 0.30% swap fee (0.25% to LPs, 0.05% protocol)
  - Slippage protection

## Architecture: v0.2.x Dependency Injection Pattern

### Service-Oriented Design

All v0.2.x programs use a **Dependency Injection (DI) architecture** with abstract service traits:

```rust
// Example: RewardDistributor trait (tribewarez-pot-o)
pub trait RewardDistributor {
    fn distribute_rewards(&self, pool_state: &mut PoolState) -> Result<()>;
}

// Two implementations:
// 1. SimpleRewardDistributor: v0.1.x compatible behavior
// 2. TensorWeightedRewardDistributor: Tensor-aware distribution
```

### ServiceRegistry Pattern

Each program includes a `ServiceRegistry` that acts as a DI container:

```rust
pub struct ServiceRegistry {
    config: ServiceConfig,
}

impl ServiceRegistry {
    pub fn create_reward_distributor(&self) -> Box<dyn RewardDistributor> {
        match self.config.use_tensor_features {
            true => Box::new(TensorWeightedRewardDistributor::new(...)),
            false => Box::new(SimpleRewardDistributor::new(...)),
        }
    }
}
```

**Benefits**:
- **Feature toggles**: Switch between v0.1.x and v0.2.0 behavior at runtime
- **Testability**: Mock implementations for unit tests
- **Maintainability**: Zero code duplication across implementations
- **Extensibility**: Add new implementations without modifying existing code

### Tensor Network Integration (REALMS Part IV)

v0.2.x introduces **quantum-inspired calculations** for enhanced reward mechanics:

#### Core Formulas (Fixed-point at 1e6 scale)

1. **Entanglement Entropy**: S = |γ| * log₂(d)
   - γ: Number of edges crossing partition cut
   - d: Bond dimension (2 for Solana constraints)
   
2. **Mutual Information**: I(A:B) = S(A) + S(B) - S(A∪B)
   - Measures information sharing between pools
   
3. **Effective Distance**: d_eff = 1 - I(A:B) / S_max
   - Determines pool decoupling for fee structures
   
4. **Coherence Probability**: P(unlock) = tanh(S_A / S_max)
   - Controls dynamic unlock conditions

#### Device Coherence Factors

Each miner/staker device has a coherence multiplier affecting:
- Reward probability
- Fee discounts
- Unlock period adjustments

| Device Type | Multiplier | Examples |
|-------------|-----------|----------|
| ASIC        | 1.0x      | Specialized mining hardware |
| GPU         | 0.8x      | NVIDIA, AMD GPUs |
| CPU         | 0.6x      | Standard processors |
| Mobile      | 0.4x      | Mobile devices |

#### Event-Driven State Changes

All state modifications emit events for off-chain tracking:

```rust
// In tribewarez-pot-o
#[event]
pub struct EntropyStateUpdated {
    pub miner: Pubkey,
    pub entropy_s: u64,
    pub mutual_info_i: u64,
    pub effective_distance: u64,
}
```

### Migration from v0.1.x

**Zero breaking changes!** Full ABI backward compatibility:

1. **Instruction signatures**: Identical to v0.1.x
2. **State layout**: New tensor fields appended (no reordering)
3. **Legacy mode**: ServiceRegistry can use v0.1.x implementations
4. **No account migration**: v0.2.0 reads v0.1.x accounts directly

**Upgrade path**:
```bash
# 1. Deploy v0.2.0 alongside v0.1.x
anchor deploy

# 2. Enable gradually per pool
solana rpc poto config --use-tensor-features true

# 3. Monitor v0.2.0 vs v0.1.x behavior
# Events help validate tensor calculations are working
```

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (v1.18+)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) (v0.30+)

## Installation

```bash
# Install Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor avm --locked
avm install latest
avm use latest

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

## Building

```bash
cd programs

# Build all programs
anchor build

# Build specific program
anchor build -p tribewarez_pot_o
anchor build -p tribewarez_staking
anchor build -p tribewarez_vault
anchor build -p tribewarez_swap
```

## Testing

```bash
# Run tests on localnet
anchor test

# Run tests with logs
anchor test -- --nocapture
```

## Deployment

### 1. Configure Network

Edit `Anchor.toml`:
```toml
[provider]
cluster = "devnet"  # or "mainnet-beta"
wallet = "~/.config/solana/id.json"
```

### 2. Generate Program Keys

```bash
# Generate new keypair for each program
solana-keygen new -o target/deploy/tribewarez_staking-keypair.json
solana-keygen new -o target/deploy/tribewarez_vault-keypair.json
solana-keygen new -o target/deploy/tribewarez_swap-keypair.json
```

### 3. Update Program IDs

After generating keypairs, update the program IDs:

```bash
# Get the new program IDs
solana-keygen pubkey target/deploy/tribewarez_staking-keypair.json
solana-keygen pubkey target/deploy/tribewarez_vault-keypair.json
solana-keygen pubkey target/deploy/tribewarez_swap-keypair.json
```

Update these addresses in:
- `tribewarez-pot-o/src/lib.rs` - `declare_id!(...)`
- `tribewarez-staking/src/lib.rs` - `declare_id!(...)`
- `tribewarez-vault/src/lib.rs` - `declare_id!(...)`
- `tribewarez-swap/src/lib.rs` - `declare_id!(...)`
- `Anchor.toml` - `[programs.*]` sections
- `src/contracts/config.ts` - Program ID constants (if present)

### 4. Deploy

```bash
# Fund deployer wallet (devnet)
solana airdrop 5 --url devnet

# Deploy all programs
anchor deploy

# Deploy specific program
anchor deploy -p tribewarez_staking
```

### 5. Initialize Programs

After deployment, initialize the programs using the admin scripts or frontend:

```typescript
// Example: Initialize staking pool
import { StakingClient, PTTC_MINT } from '../contracts';

const client = new StakingClient(connection);
// ... initialize pool with admin wallet
```

## Program Architecture

### Service Traits by Program

**tribewarez-pot-o**:
- `ProofValidator`: Validate PoT-O mining proofs
- `MinerManager`: Manage miner lifecycle and entropy
- `RewardDistributor`: Calculate and distribute rewards
- `TensorPoolService`: Calculate entropy and mutual information

**tribewarez-staking**:
- `StakingCalculator`: Compute time-based rewards with entropy factors
- `EntanglementService`: Track pool coupling and unlock probability

**tribewarez-vault**:
- `VaultSecurityProvider`: Manage dynamic locktimes and early withdrawal fees

**tribewarez-swap**:
- `SwapCalculator`: Compute AMM prices with coherence fee adjustments

### Account Structure

**v0.2.0 State Layout** (ABI compatible with v0.1.x):

```
PotOConfig (v0.1.x fields)
├── authority, min_difficulty, max_difficulty, ...
├── [NEW v0.2.0] entropy_weight: u64
├── [NEW v0.2.0] mutual_info_scale: u64
├── [NEW v0.2.0] device_coherence_factors: [u64; 4]
└── [RESERVED] padding: [u8; 256] for future expansion

MinerAccount (v0.1.x fields)
├── owner, total_hashes, pool_shares, ...
├── [NEW v0.2.0] entropy_s: u64
├── [NEW v0.2.0] mutual_info_i: u64
├── [NEW v0.2.0] device_coherence: u64
└── [RESERVED] padding: [u8; 256] for future expansion

ProofRecord (v0.1.x fields)
├── miner, proof_hash, difficulty, ...
├── [NEW v0.2.0] entropy_contribution: u64
└── [RESERVED] padding: [u8; 128] for future expansion
```

**Key Design**:
- New fields always appended (no existing field reordering)
- Padding reserved for future expansion (no re-deployment needed)
- v0.1.x clients can read up to their expected struct size
- v0.2.0 clients read all fields, using defaults for v0.1.x accounts

### Security Features

- **PDA-based account derivation**: Deterministic addresses prevent collisions
- **Authority checks**: Only authorized signers can modify state
- **Overflow protection**: All math uses checked operations
- **Reentrancy protection**: State updates before CPI calls
- **Time-lock enforcement**: Cannot unstake before unlock time

## Token Configuration

**PTtC (Pumped TRIBE-Test Coin)**
- Mint: `BikceVyDGWMNUTNhSKo789ThWZRfLr2q9TJYc4bLpump`
- Decimals: 6
- Network: Solana Mainnet (Pump.fun)

## Frontend Integration

The frontend SDK is located in `src/contracts/`:

```typescript
import { 
  StakingClient, 
  SwapClient, 
  PTTC_MINT,
  createStakingClient 
} from './contracts';

// Create client
const stakingClient = createStakingClient(connection);

// Stake tokens
const instructions = await stakingClient.createStakeInstruction(
  userPublicKey,
  parseTokenAmount(100, 6), // 100 PTtC
  PTTC_MINT
);
```

React hooks are available in `src/hooks/`:
- `useStaking()` - Staking operations
- `useSwap()` - Token swap operations

## Environment Variables

```env
VITE_RPC_ENDPOINT=https://api.mainnet-beta.solana.com
VITE_RPC_ENDPOINT_API_KEY=your-helius-api-key
```

## Publishing (crates.io)

### v0.2.x Release

All program crates are published to crates.io as part of the v0.2 line:
- `tribewarez-pot-o` v0.2.0
- `tribewarez-staking` v0.2.1
- `tribewarez-vault` v0.2.0
- `tribewarez-swap` v0.2.0
- `pot-o-core` v0.2.0 (provides tensor types and calculations)

**Installation**:
```toml
[dependencies]
tribewarez-pot-o = "0.2.0"
tribewarez-staking = "0.2.1"
tribewarez-vault = "0.2.0"
tribewarez-swap = "0.2.0"
pot-o-core = "0.2.0"
```

See [CHANGELOG.md](CHANGELOG.md) for v0.2.x features, testing results, and migration guide from v0.1.x.

## Support

- Website: https://tribewarez.com
- DeFi Portal: https://defi.tribewarez.com
- Token: https://pump.fun/BikceVyDGWMNUTNhSKo789ThWZRfLr2q9TJYc4bLpump

## License

MIT License - See [LICENSE](LICENSE) file for details.
