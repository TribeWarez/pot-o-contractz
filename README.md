# Tribewarez DeFi Smart Contracts

[![CI](https://img.shields.io/github/actions/workflow/status/TribeWarez/pot-o-contractz/ci.yml?branch=main&label=CI)](https://github.com/TribeWarez/pot-o-contractz/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/actions/workflow/status/TribeWarez/pot-o-contractz/release.yml?label=Release)](https://github.com/TribeWarez/pot-o-contractz/actions/workflows/release.yml)
[![Auto Tag](https://img.shields.io/github/actions/workflow/status/TribeWarez/pot-o-contractz/tag.yml?branch=main&label=Auto+Tag)](https://github.com/TribeWarez/pot-o-contractz/actions/workflows/tag.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Solana programs (smart contracts) for the Tribewarez DeFi platform supporting PTtC (Pumped TRIBE-Test Coin) operations.

**v0.3.x**: PoT-O Contractz - Token ecosystem programs with governance, bridging, routing, and liquidity pools.

## Programs Overview

### Core Programs (v0.2.x - Tensor Network Integration)

#### 0. PoT-O Program (`tribewarez-pot-o`)
- **Purpose**: On-chain validation of Proof of Tensor Optimizations mining proofs
- **Features**:
  - Config and miner registration
  - Submit and validate PoT-O proofs (MML threshold, path distance, computation hash)
  - Reward distribution and difficulty adjustment
  - Claim rewards (TW-RPC-001 aligned)

#### 1. Staking Program (`tribewarez-staking`)
- **Purpose**: Stake PTtC tokens to earn rewards over time
- **Features**:
  - Flexible and time-locked staking pools
  - Configurable reward rates (APY)
  - Compound rewards
  - Emergency unstake (forfeit rewards)
  - Admin pool management
  - Tensor entanglement for cooperative staking

#### 2. Vault Program (`tribewarez-vault`)
- **Purpose**: Secure token storage and escrow functionality
- **Features**:
  - Personal vaults for users
  - Time-locked savings accounts
  - Two-party escrow with release conditions
  - Deposit/withdrawal with audit trail
  - Dynamic locktime reduction based on network activity

#### 3. Swap Program (`tribewarez-swap`)
- **Purpose**: Automated Market Maker (AMM) for token swaps
- **Features**:
  - Constant product formula (x * y = k)
  - Liquidity provision with LP tokens
  - 0.30% swap fee (0.25% to LPs, 0.05% protocol)
  - Slippage protection
  - Tensor-aware dynamic fee discounts

### PoT-O Contractz Programs (v0.3.x)

#### 4. Tokens Program (`tribewarez-tokens`)
- **Purpose**: Multi-token ecosystem management (AUMCOIN, TRIBECOIN, RAVECOIN)
- **Features**:
  - Token minting with supply caps
  - Inflation rate configuration
  - Burning and transfers
  - Freeze/thaw account capabilities
  - Metadata management (name, symbol, URI)

#### 5. Governance Program (`tribewarez-governance`)
- **Purpose**: DAO operations and proposal management
- **Features**:
  - Create and manage proposals
  - Voting (For, Against, Abstain)
  - Proposal execution with timelock
  - Treasury management
  - Quorum requirements

#### 6. Bridge Program (`tribewarez-bridge`)
- **Purpose**: Cross-chain token wrapping (NMTC/PPTC)
- **Features**:
  - Token deposit and withdrawal
  - Wrapped token minting
  - Pause/unpause functionality
  - Admin-controlled bridge management

#### 7. Router Program (`tribewarez-router`)
- **Purpose**: Token swap routing and price queries
- **Features**:
  - Multi-hop swaps
  - Price impact calculation
  - Slippage tolerance validation
  - Quote generation for frontend

#### 8. Liquidity Program (`tribewarez-liquidity`)
- **Purpose**: AMM-style liquidity pool management
- **Features**:
  - Create token pairs
  - Add/remove liquidity
  - Swap with dynamic fees
  - TWAP price feeds
  - Pool position tracking

## Architecture: v0.3.x Multi-Program Design

### Token Ecosystem Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Tribewarez DeFi                          │
├─────────────────────────────────────────────────────────────┤
│  Governance          │  Bridge          │  Router          │
│  (tribewarez-        │  (tribewarez-   │  (tribewarez-    │
│   governance)        │   bridge)        │   router)        │
├─────────────────────────────────────────────────────────────┤
│  Tokens (tribewarez-tokens)                                │
│  - AUMCOIN    - TRIBECOIN    - RAVECOIN                   │
├─────────────────────────────────────────────────────────────┤
│  Liquidity (tribewarez-liquidity)                          │
│  - Pool pairs  - LP tokens   - Swap logic                 │
├─────────────────────────────────────────────────────────────┤
│  Core (v0.2.x)                                             │
│  - PoT-O   - Staking   - Vault   - Swap                   │
└─────────────────────────────────────────────────────────────┘
```

### Dependency Injection Pattern (v0.2.x)

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

Each v0.2.x program includes a `ServiceRegistry` that acts as a DI container:

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

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (v1.18+)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) (v0.30+)

## Building

```bash
# Build all programs
cargo build

# Build specific program
cargo build -p tribewarez_pot_o
cargo build -p tribewarez_staking
cargo build -p tribewarez_vault
cargo build -p tribewarez_swap
cargo build -p tribewarez_tokens
cargo build -p tribewarez_governance
cargo build -p tribewarez_bridge
cargo build -p tribewarez_router
cargo build -p tribewarez_liquidity
```

## Testing

```bash
# Run all tests
cargo test

# Run tests with logs
cargo test -- --nocapture

# Run tests for specific program
cargo test -p tribewarez_tokens
```

## Linting

```bash
# Run clippy with strict warnings (CI mode)
cargo clippy -- -D warnings

# Check formatting
cargo fmt --check
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
solana-keygen new -o target/deploy/tribewarez_pot_o-keypair.json
solana-keygen new -o target/deploy/tribewarez_staking-keypair.json
solana-keygen new -o target/deploy/tribewarez_vault-keypair.json
solana-keygen new -o target/deploy/tribewarez_swap-keypair.json
solana-keygen new -o target/deploy/tribewarez_tokens-keypair.json
solana-keygen new -o target/deploy/tribewarez_governance-keypair.json
solana-keygen new -o target/deploy/tribewarez_bridge-keypair.json
solana-keygen new -o target/deploy/tribewarez_router-keypair.json
solana-keygen new -o target/deploy/tribewarez_liquidity-keypair.json
```

### 3. Update Program IDs

After generating keypairs, update the program addresses in each program's `src/lib.rs`:
- `tribewarez-pot-o/src/lib.rs` - `declare_id!(...)`
- `tribewarez-staking/src/lib.rs` - `declare_id!(...)`
- `tribewarez-vault/src/lib.rs` - `declare_id!(...)`
- `tribewarez-swap/src/lib.rs` - `declare_id!(...)`
- `tribewarez-tokens/src/lib.rs` - `declare_id!(...)`
- `tribewarez-governance/src/lib.rs` - `declare_id!(...)`
- `tribewarez-bridge/src/lib.rs` - `declare_id!(...)`
- `tribewarez-router/src/lib.rs` - `declare_id!(...)`
- `tribewarez-liquidity/src/lib.rs` - `declare_id!(...)`

### 4. Deploy

```bash
# Fund deployer wallet (devnet)
solana airdrop 5 --url devnet

# Deploy all programs
anchor deploy
```

## Publishing (crates.io)

### v0.3.x Release

All program crates are published to crates.io as part of the v0.3 line:

| Crate | Version | Description |
|-------|---------|-------------|
| tribewarez-pot-o | 0.3.2 | PoT-O mining validation |
| tribewarez-staking | 0.3.2 | PTtC staking with tensor features |
| tribewarez-vault | 0.3.2 | Vault and escrow |
| tribewarez-swap | 0.3.2 | AMM swap |
| tribewarez-tokens | 0.3.2 | Multi-token management |
| tribewarez-governance | 0.3.2 | DAO governance |
| tribewarez-bridge | 0.3.2 | Token bridging |
| tribewarez-router | 0.3.2 | Swap routing |
| tribewarez-liquidity | 0.3.2 | Liquidity pools |

**Installation**:
```toml
[dependencies]
tribewarez-pot-o = "0.3.2"
tribewarez-staking = "0.3.2"
tribewarez-vault = "0.3.2"
tribewarez-swap = "0.3.2"
tribewarez-tokens = "0.3.2"
tribewarez-governance = "0.3.2"
tribewarez-bridge = "0.3.2"
tribewarez-router = "0.3.2"
tribewarez-liquidity = "0.3.2"
```

## Token Configuration

**PTtC (Pumped TRIBE-Test Coin)**
- Mint: `BikceVyDGWMNUTNhSKo789ThWZRfLr2q9TJYc4bLpump`
- Decimals: 6
- Network: Solana Mainnet (Pump.fun)

## Support

- Website: https://tribewarez.com
- DeFi Portal: https://defi.tribewarez.com
- Token: https://pump.fun/BikceVyDGWMNUTNhSKo789ThWZRfLr2q9TJYc4bLpump

## License

MIT License - See [LICENSE](LICENSE) file for details.
