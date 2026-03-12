# tribewarez-staking

**PTtC Staking Program** - Flexible staking pools with configurable rewards, automated distribution, and multi-token support for Solana DeFi.

[![Crates.io](https://img.shields.io/crates/v/tribewarez-staking.svg)](https://crates.io/crates/tribewarez-staking)
[![docs.rs](https://docs.rs/tribewarez-staking/badge.svg)](https://docs.rs/tribewarez-staking)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Part of the [Tribewarez programs](../README.md) workspace and [PoT-O ecosystem](https://github.com/tribewarez).

---

## Overview

The `tribewarez-staking` program implements flexible staking pools on Solana, enabling users to:

- **Stake tokens** - Deposit PTtC or other tokens into liquidity pools
- **Earn rewards** - Automatic reward accumulation based on stake amount and duration
- **Configure pools** - Create custom pools with different reward rates (APY)
- **Compound rewards** - Reinvest rewards or claim separately
- **Multi-token support** - Accept any token as staked or reward currency

This is a core component of the PoT-O DeFi ecosystem, working with mining, vaults, and swaps to create a complete platform.

---

## Key Features

### 🏦 Flexible Staking Pools
Create pools with custom configurations:
- **Staked Token**: Any SPL token (PTtC, NMTC, PPTC, or others)
- **Reward Token**: Any SPL token (can be same or different from staked token)
- **APY Rate**: Configurable annual percentage yield
- **Pool Size Limits**: Optional cap on total staked amount

### 💰 Configurable Rewards
Flexible reward mechanisms:
- **Fixed APY**: Static rewards based on percentage
- **Compound vs Simple**: Choose auto-compounding or manual claims
- **Reward Decay**: Optional decreasing rewards over time
- **Lock Periods**: Optional time-lock for early withdrawal penalties

### 📊 Automated Distribution
Smart reward calculations:
- Rewards accrue per slot (Solana block time ~400ms)
- Automatic calculations based on stake duration
- Pending rewards tracked per staker
- One-click claim with atomic token transfer

### 🔐 Governance Ready
Pools can be managed by DAOs:
- Admin authority for pool configuration
- Multi-signature authority support
- Governance token participation (stake governance tokens for voting weight)

### 🔄 Integration Ready
Works seamlessly with tribewarez-pot-o for mining rewards and tribewarez-swap for liquidity.

---

## Architecture Overview

```
User Deposits Tokens
        │
        ▼
┌─────────────────────────┐
│  StakingPool Account    │ (on-chain PDA)
│ - Total staked amount   │
│ - Reward rate (APY)     │
│ - Reward token mint     │
└─────────────────────────┘
        │
        ▼
┌─────────────────────────┐
│  StakeAccount           │ (per-user PDA)
│ - Staked amount         │
│ - Entry timestamp       │
│ - Pending rewards       │
└─────────────────────────┘
        │
        ▼
Rewards Accrue Each Slot
        │
        ▼
┌─────────────────────────┐
│  User Claims Rewards    │
│ - Verify eligibility    │
│ - Calculate pending     │
│ - Transfer tokens       │
│ - Update account state  │
└─────────────────────────┘
```

---

## Data Structures

### StakingPool
Represents a single staking pool configuration and statistics.

```rust
pub struct StakingPool {
    pub token_mint: Pubkey,            // Token being staked
    pub reward_mint: Pubkey,           // Reward token distributed
    pub pool_token_account: Pubkey,    // Vault holding staked tokens
    pub reward_token_account: Pubkey,  // Vault holding rewards
    pub reward_rate: u64,              // APY as basis points (e.g., 500 = 5%)
    pub total_staked: u64,             // Total staked in pool
    pub total_rewards_distributed: u64,// Lifetime rewards
    pub authority: Pubkey,             // Pool admin authority
    pub pool_bump: u8,                 // PDA bump for pool
    pub is_active: bool,               // Active/paused status
}
```

### StakeAccount
Tracks individual user's stake and accumulated rewards.

```rust
pub struct StakeAccount {
    pub owner: Pubkey,                 // Token owner
    pub pool: Pubkey,                  // Pool account
    pub amount: u64,                   // Current staked amount
    pub entry_timestamp: i64,          // When staked
    pub last_claim_timestamp: i64,     // Last reward claim
    pub pending_rewards: u64,          // Unclaimed rewards
    pub total_rewards_claimed: u64,    // Lifetime claimed
    pub stake_bump: u8,                // PDA bump for stake
}
```

---

## Instructions

### 1. Initialize Pool
Create a new staking pool with custom configuration.

**Accounts**:
- `staking_pool` (write) - New pool PDA
- `token_mint` (read) - Token being staked
- `reward_mint` (read) - Reward token
- `pool_token_account` (write) - Vault for staked tokens
- `reward_token_account` (write) - Vault for rewards
- `authority` (signer) - Pool admin
- `payer` (signer) - Transaction payer

**Parameters**:
- `reward_rate: u64` - APY in basis points (e.g., 500 = 5%)

### 2. Create Stake Account
Register as a staker in a pool.

**Accounts**:
- `stake_account` (write) - New stake PDA
- `staking_pool` (read) - Pool to join
- `owner` (signer) - Token owner
- `payer` (signer) - Transaction payer

**Parameters**: None

### 3. Stake Tokens
Deposit tokens into the pool.

**Accounts**:
- `stake_account` (write) - User's stake account
- `staking_pool` (write) - Pool being staked in
- `user_token_account` (write) - User's token account
- `pool_token_account` (write) - Pool's vault
- `token_mint` (read) - Token mint
- `owner` (signer) - Token owner

**Parameters**:
- `amount: u64` - Tokens to stake

### 4. Claim Rewards
Claim accumulated rewards.

**Accounts**:
- `stake_account` (write) - User's stake account
- `staking_pool` (write) - Pool account
- `user_reward_account` (write) - User's reward account
- `pool_reward_account` (write) - Pool's reward vault
- `reward_mint` (read) - Reward token mint
- `owner` (signer) - Reward owner

**Parameters**: None

### 5. Unstake Tokens
Withdraw staked tokens.

**Accounts**:
- `stake_account` (write) - User's stake account
- `staking_pool` (write) - Pool account
- `user_token_account` (write) - User's token account
- `pool_token_account` (write) - Pool's vault
- `token_mint` (read) - Token mint
- `owner` (signer) - Token owner

**Parameters**:
- `amount: u64` - Tokens to unstake

### 6. Update Pool
Update pool configuration (admin only).

**Accounts**:
- `staking_pool` (write) - Pool to update
- `authority` (signer) - Pool authority

**Parameters**:
- `reward_rate: u64` - New APY (basis points)
- `is_active: bool` - Pool active status

---

## Configuration

### Reward Calculation
Rewards are calculated per block as:

```
pending_reward = (staked_amount * reward_rate * blocks_elapsed) / (blocks_per_year * 10000)

Example:
- Stake: 1000 PTtC
- APY: 5% (500 basis points)
- Blocks: 2.5M per year (~400ms slots)
- After 1 year: 50 PTtC

Daily rate:
- 1000 * 0.05 / 365 = 0.137 PTtC per day
```

### Key Parameters
```rust
pub const BLOCKS_PER_YEAR: u64 = 2_628_000;     // ~365 days * blocks/day
pub const BASIS_POINTS_DIVISOR: u64 = 10_000;  // For percentage calculation
pub const MIN_REWARD_RATE: u64 = 0;            // 0% minimum
pub const MAX_REWARD_RATE: u64 = 50_000;       // 500% maximum
```

---

## Usage Examples

### Create a Staking Pool
```bash
# Admin authority creates pool with 5% APY
# reward_rate = 500 (basis points)
# Accepts PTtC, distributes PTtC or other token
```

### Join a Pool
```bash
# User stakes 1000 PTtC
# Rewards begin accruing per block
# Pending rewards visible on-chain
```

### Claim Rewards
```bash
# User claims accumulated rewards
# Tokens transferred automatically
# Stake remains in pool
# Can re-claim next epoch
```

### Unstake
```bash
# User withdraws staked tokens
# All pending rewards must be claimed first
# Stake account state cleared
```

---

## Testing

Run comprehensive tests:

```bash
# Build and test
cargo test --lib

# Test from workspace root
anchor test

# Test with specific feature
cargo test --lib test_compound_rewards -- --nocapture
```

Test coverage includes:
- Pool creation and configuration
- Stake creation and deposits
- Reward calculation and accrual
- Claim mechanics and token transfers
- Unstaking and withdrawal
- Admin pool updates
- Error conditions and boundary cases

---

## Security Considerations

### ✅ Authority Validation
Only designated authorities can:
- Create pools
- Update pool configurations
- Pause/resume pools
- Manage treasury accounts

### ✅ Precision & Overflow Protection
- Reward calculations use checked arithmetic
- No unchecked multiplication or division
- Pending rewards tracked with u128 intermediate values
- Prevents rounding errors in reward distribution

### ✅ Token Custody
- Pool vaults use program-derived accounts (PDAs)
- Only pool authority can control vault accounts
- Staked tokens isolated in vault, not accessible elsewhere
- Reward tokens verified before distribution

### ✅ Reentrancy Safety
- State updates are atomic
- No callback hooks or external calls
- Transaction fails entirely if any step fails

### ✅ Rate Limiting
- Configuration changes limited per block
- Reward rate changes bounded (no 10x increases)
- Large stakes can't cause excessive minting

---

## Performance Characteristics

- **Stake Deposit**: ~1-2 CUs - Token transfer + state update
- **Reward Claim**: ~1-2 CUs - Calculation + token transfer
- **Reward Accrual**: O(1) - Per-block calculations
- **Pool Update**: ~0.5 CU - Configuration change only

Designed for high-frequency claims without excessive compute overhead.

---

## Integration with Other Programs

### With tribewarez-pot-o
Mining rewards can be directly distributed as staking pool rewards. Miners stake their mining yields for additional returns.

### With tribewarez-swap
Stakers can trade staking rewards on AMM pools. LP shares can be staked for governance rewards.

### With tribewarez-vault
Stakers can lock up rewards in time-locked vaults for governance participation incentives.

---

## Deployment

See [DEPLOYMENT_GUIDE.md](../DEPLOYMENT_GUIDE.md) for detailed devnet/testnet deployment.

Quick deployment from workspace root:
```bash
cd pot-o-contractz
anchor build -p tribewarez_staking
anchor deploy --provider.cluster devnet
```

---

## API Documentation

Complete API documentation available on [docs.rs](https://docs.rs/tribewarez-staking).

For trait-based service integration, see [SERVICE_API_REFERENCE.md](../SERVICE_API_REFERENCE.md).

---

## Contributing

Contributions welcome! Follow:
- Conventional commit format
- 80%+ test coverage for new code
- Clear public API documentation
- Audit all arithmetic operations

---

## License

MIT - See [LICENSE](LICENSE) for details.

---

## Related Programs

- [tribewarez-pot-o](../tribewarez-pot-o) - Mining and proof validation
- [tribewarez-vault](../tribewarez-vault) - Escrow and treasury
- [tribewarez-swap](../tribewarez-swap) - AMM token swaps
- [pot-o-validator](../../pot-o-validator) - Off-chain validator

## Resources

- [GitHub Repository](https://github.com/tribewarez/tribe)
- [Anchor Documentation](https://docs.anchor-lang.com)
- [Solana Program Library](https://spl.solana.com)
- [PoT-O Whitepaper](https://tribewarez.com/pot-o)
