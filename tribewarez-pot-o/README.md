# tribewarez-pot-o

**PoT-O (Proof of Tensor Optimizations)** - On-chain Solana program for validating tensor-based computational proofs and distributing mining rewards.

[![Crates.io](https://img.shields.io/crates/v/tribewarez-pot-o.svg)](https://crates.io/crates/tribewarez-pot-o)
[![docs.rs](https://docs.rs/tribewarez-pot-o/badge.svg)](https://docs.rs/tribewarez-pot-o)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Part of the [Tribewarez programs](../README.md) workspace and [PoT-O ecosystem](https://github.com/tribewarez).

---

## Overview

The `tribewarez-pot-o` program implements Proof of Tensor Optimizations on Solana, enabling miners to submit computational proofs that are validated on-chain. It manages:

- **Miner registration** - Track and authorize miners
- **Proof submission** - Accept computational proof submissions
- **Proof validation** - Verify tensor network computations on-chain
- **Reward distribution** - Award tokens to successful miners
- **Difficulty adjustment** - Maintain target block rate

This is the core mining program in the PoT-O ecosystem, working alongside staking, vault, and swap programs for a complete DeFi platform.

---

## Key Features

### ✨ Proof of Tensor Optimizations
Validators run tensor network computations offline and submit proofs on-chain. The program verifies the computation was performed correctly using cryptographic commitments and merkle proofs.

### 🔐 Secure Miner Registration
Register miners on-chain with authority validation. Only registered miners can submit valid proofs. Support for multiple authority signatures for governance.

### 💰 Configurable Reward Distribution
Flexible reward structure supporting multiple token types. Rewards can be:
- Distributed per proof validation
- Adjusted via governance
- Split among protocol, stakers, and miners

### 📊 Dynamic Difficulty Adjustment
Automatic difficulty adjustment based on submission rate to maintain consistent block validation times (~2.5 minutes per proof).

### 🔄 Integration Ready
Works seamlessly with tribewarez-staking for reward distribution and tribewarez-swap for liquidity provision.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│          Miner Submits Proof Off-Chain              │
└─────────────────┬───────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────┐
│   Miner Account Lookup & Validation (on-chain)      │
│   - Check miner is registered                       │
│   - Check authority signature                       │
└─────────────────┬───────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────┐
│   Tensor Network Proof Validation (on-chain)        │
│   - Verify computation hash                         │
│   - Check merkle proof path                         │
│   - Validate against current difficulty             │
└─────────────────┬───────────────────────────────────┘
                  │
                  ▼ Success
┌─────────────────────────────────────────────────────┐
│   Reward Distribution                               │
│   - Mint/transfer reward tokens                     │
│   - Update miner statistics                         │
│   - Record block (for difficulty adjustment)        │
└─────────────────────────────────────────────────────┘
```

---

## Data Structures

### MinerAccount
Tracks individual miner information and statistics.

```rust
pub struct MinerAccount {
    pub authority: Pubkey,           // Miner's authority account
    pub total_proofs_submitted: u64, // Lifetime proof count
    pub total_proofs_valid: u64,     // Valid proofs accepted
    pub total_rewards: u64,          // Total rewards claimed
    pub is_active: bool,             // Active status
    pub bump: u8,                    // PDA bump seed
}
```

### DifficultyState
Global state for difficulty adjustment and mining statistics.

```rust
pub struct DifficultyState {
    pub current_difficulty: u64,     // Current target difficulty
    pub last_adjustment_slot: u64,   // Last difficulty adjustment
    pub target_block_time: u64,      // Target time in slots
    pub total_blocks_validated: u64, // Lifetime block count
    pub average_block_time: u64,     // Moving average
    pub bump: u8,                    // PDA bump seed
}
```

---

## Instructions

### 1. Initialize Program
Sets up difficulty state and initializes program authority.

**Accounts**:
- `difficulty_state` (write) - Global difficulty tracking
- `payer` (signer) - Transaction payer

**Parameters**: None

### 2. Register Miner
Register a new miner account.

**Accounts**:
- `miner_account` (write) - New miner PDA
- `authority` (signer) - Miner's authority
- `payer` (signer) - Transaction payer

**Parameters**: None

### 3. Submit Proof
Submit a tensor computation proof for validation.

**Accounts**:
- `miner_account` (read) - Miner's account
- `difficulty_state` (read) - Current difficulty
- `authority` (signer) - Miner's authority signature

**Parameters**:
- `computation_hash: [u8; 32]` - Hash of computation result
- `proof_path: Vec<[u8; 32]>` - Merkle proof path
- `nonce: u64` - Proof nonce

### 4. Claim Rewards
Claim accumulated rewards from validated proofs.

**Accounts**:
- `miner_account` (write) - Miner's account
- `reward_token_account` (write) - Miner's reward token account
- `token_mint` (read) - Reward token mint
- `authority` (signer) - Miner's authority

**Parameters**:
- `amount: u64` - Reward amount to claim

### 5. Adjust Difficulty
Update difficulty based on recent block times.

**Accounts**:
- `difficulty_state` (write) - Difficulty state
- `authority` (signer) - Program authority (DAO governance)

**Parameters**:
- `new_difficulty: u64` - New difficulty target

---

## Configuration

### Difficulty Parameters
```rust
pub const TARGET_BLOCK_TIME: u64 = 150;        // ~2.5 minutes in slots
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 2016; // Blocks
pub const MAX_DIFFICULTY_INCREASE: f64 = 1.25; // 25% max per adjustment
pub const MAX_DIFFICULTY_DECREASE: f64 = 0.75; // 25% max per adjustment
```

### Reward Structure
```rust
pub const REWARD_PER_VALID_PROOF: u64 = 10_000_000; // Amount in smallest units
pub const REWARD_DECAY_RATE: f64 = 0.995;         // Per block decay
```

---

## Usage Examples

### Register as a Miner
```rust
// Off-chain: Create your authority keypair
let authority = Keypair::new();

// On-chain: Call register_miner instruction
// This creates your MinerAccount PDA
```

### Submit a Proof
```rust
// Off-chain: Compute tensor network and generate proof
let computation = compute_tensor_network(data);
let proof_hash = keccak256(computation);
let merkle_proof = generate_merkle_proof(proof_hash);

// On-chain: Submit proof
// Program validates proof against current difficulty
// If valid, rewards are awarded to your account
```

### Claim Rewards
```rust
// On-chain: Retrieve accumulated rewards
// Tokens are transferred to your reward token account
// Miner statistics are updated
```

---

## Testing

Run tests locally:

```bash
# Build and test
cargo test --lib

# Test from workspace root
anchor test

# Test with logging
RUST_LOG=debug cargo test -- --nocapture
```

Test coverage includes:
- Miner registration and authority checks
- Proof submission and validation
- Difficulty adjustment calculations
- Reward distribution and claims
- Error handling for invalid proofs

---

## Security Considerations

### ✅ Authority Validation
All instructions that modify state require proper authority signatures. Miners can only interact with their own accounts.

### ✅ Proof Verification
Proofs are validated using cryptographic commitments. A valid proof demonstrates computation was actually performed (not just guessed).

### ✅ Difficulty Protection
Difficulty adjustment is bounded to prevent rapid changes. Gradual adjustments maintain mining incentives.

### ✅ Reward Limits
Reward amounts are pre-configured and validated before distribution. No unchecked minting or transfers.

### ✅ State Consistency
Miner statistics are updated atomically with reward distribution, preventing state inconsistencies.

---

## Performance Characteristics

- **Proof Submission**: ~0.5 CUs (compute units) - minimal on-chain computation
- **Reward Distribution**: ~1-2 CUs - simple token transfer
- **Difficulty Adjustment**: ~0.5 CUs - arithmetic only
- **Total Per Block**: ~2-3 CUs out of 1.4M available

Proof validation work is done off-chain by validators, making on-chain operations very efficient.

---

## Integration with Other Programs

### With tribewarez-staking
Miners can delegate staking rewards to staking pools. Rewards from PoT-O are compatible with pool reward token configuration.

### With tribewarez-swap
Miners can trade rewards on the AMM. Liquidity pools support reward token pairs.

### With tribewarez-vault
Miners can lock up rewards in vaults with time-locks for governance participation.

---

## Deployment

See [DEPLOYMENT_GUIDE.md](../DEPLOYMENT_GUIDE.md) for detailed devnet/testnet deployment instructions.

Quick deployment from workspace root:
```bash
cd pot-o-contractz
anchor build -p tribewarez_pot_o
anchor deploy --provider.cluster devnet
```

---

## API Documentation

Complete API documentation is available on [docs.rs](https://docs.rs/tribewarez-pot-o).

For trait-based service integration, see [SERVICE_API_REFERENCE.md](../SERVICE_API_REFERENCE.md).

---

## Contributing

Contributions welcome! Please follow:
- Conventional commit messages (feat:, fix:, docs:, test:)
- 80%+ test coverage for new code
- Clear documentation for public APIs
- Security audit for state-modifying code

---

## License

MIT - See [LICENSE](LICENSE) for details.

---

## Related Programs

- [tribewarez-staking](../tribewarez-staking) - Staking pool management
- [tribewarez-vault](../tribewarez-vault) - Escrow and treasury
- [tribewarez-swap](../tribewarez-swap) - AMM token swaps
- [pot-o-validator](../../pot-o-validator) - Off-chain validator daemon

## Resources

- [GitHub Repository](https://github.com/tribewarez/tribe)
- [Anchor Documentation](https://docs.anchor-lang.com)
- [Solana Program Library](https://spl.solana.com)
- [PoT-O Whitepaper](https://tribewarez.com/pot-o)
