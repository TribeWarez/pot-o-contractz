# Tribewarez DeFi Smart Contracts

Solana programs (smart contracts) for the Tribewarez DeFi platform supporting PTtC (Pumped TRIBE-Test Coin) operations.

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

### Account Structure

```
StakingPool (PDA: ["staking_pool", token_mint])
├── authority: Pubkey
├── token_mint: Pubkey (PTtC)
├── reward_mint: Pubkey
├── pool_token_account: Pubkey
├── reward_rate: u64
├── lock_duration: i64
├── total_staked: u64
└── is_active: bool

StakeAccount (PDA: ["stake", pool, user])
├── owner: Pubkey
├── amount: u64
├── stake_time: i64
├── unlock_time: i64
├── pending_rewards: u64
└── total_rewards_claimed: u64
```

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

## Support

- Website: https://tribewarez.com
- DeFi Portal: https://defi.tribewarez.com
- Token: https://pump.fun/BikceVyDGWMNUTNhSKo789ThWZRfLr2q9TJYc4bLpump

## License

MIT License - See LICENSE file for details.
