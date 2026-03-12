# tribewarez-vault

**Vault & Escrow** - Secure token storage, time-locked savings, and conditional escrow agreements for Tribewarez DeFi.

[![Crates.io](https://img.shields.io/crates/v/tribewarez-vault.svg)](https://crates.io/crates/tribewarez-vault)
[![docs.rs](https://docs.rs/tribewarez-vault/badge.svg)](https://docs.rs/tribewarez-vault)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Part of the [Tribewarez programs](../README.md) workspace and [PoT-O ecosystem](https://github.com/tribewarez).

---

## Overview

The `tribewarez-vault` program provides secure token storage and escrow functionality on Solana. It enables users to:

- **Create personal vaults** - Private token storage accounts with optional time-locks
- **Time-locked savings** - Lock tokens until a specified timestamp
- **Escrow agreements** - Two-party conditional token release
- **Treasury management** - Centralized vault administration with activity tracking
- **Lock extension** - Extend existing vault lock periods (cannot reduce)

This program works alongside the PoT-O mining, staking, and swap programs to provide a complete DeFi ecosystem.

---

## Key Features

### 🔒 Time-Locked Vaults
Deposit tokens and set a lock period. Tokens cannot be withdrawn until the lock expires, enforcing commitment to long-term holdings.

### 🤝 Conditional Escrow
Create escrow agreements where tokens are held by the program until a release condition is met (time-based). Either party can interact with the escrow state.

### 💼 Treasury Management
Central treasury account tracks aggregate vault statistics:
- Total tokens deposited across all vaults
- Number of active vaults
- Admin controls for emergency situations

### 🔐 Authority-Based Control
Each vault and escrow has clear ownership and authorization rules. Only authorized parties can trigger state changes.

### ⏰ Temporal Enforcement
All lock periods enforced using Solana's Clock sysvar for consistent, tamper-proof time tracking.

---

## Architecture Overview

```
┌──────────────────────────────────────────────────┐
│       User Initiates Vault / Escrow Action       │
└──────────────────┬───────────────────────────────┘
                   │
        ┌──────────┴────────────┐
        │                       │
        ▼                       ▼
  ┌───────────────┐      ┌──────────────────┐
  │  Create Vault │      │  Create Escrow   │
  │               │      │                  │
  │ - Set owner   │      │ - Set depositor  │
  │ - Set lock    │      │ - Set beneficiary│
  │ - Init state  │      │ - Set release    │
  └───────┬───────┘      │   time           │
          │              └────────┬─────────┘
          │                       │
          ▼                       ▼
  ┌───────────────┐      ┌──────────────────┐
  │  Deposit      │      │  Release Escrow  │
  │               │      │  (after time)    │
  │ - Transfer in │      │                  │
  │ - Update bal  │      │ - Check time     │
  │ - Log event   │      │ - Transfer out   │
  └───────┬───────┘      │ - Mark released  │
          │              └────────┬─────────┘
          │                       │
          └──────────┬────────────┘
                     │
                     ▼
            ┌──────────────────┐
            │   Withdraw       │
            │                  │
            │ - Check lock     │
            │ - Transfer out   │
            │ - Update stats   │
            └──────────────────┘
```

---

## Data Structures

### Treasury
Central management account for all vaults and escrows.

```rust
pub struct Treasury {
    pub authority: Pubkey,           // Admin authority
    pub token_mint: Pubkey,          // Token this treasury manages
    pub vault_token_account: Pubkey, // Account holding all tokens
    pub total_deposited: u64,        // Sum of all deposits
    pub total_vaults: u64,           // Count of active vaults
    pub bump: u8,                    // PDA bump seed
    pub is_active: bool,             // Accept new deposits
    pub created_at: i64,             // Creation timestamp
}
```

### UserVault
Individual time-locked vault for a user.

```rust
pub struct UserVault {
    pub owner: Pubkey,          // Vault owner
    pub treasury: Pubkey,       // Parent treasury
    pub name: String,           // Human-readable name (≤32 chars)
    pub balance: u64,           // Current token balance
    pub lock_until: i64,        // Unix timestamp when withdrawable
    pub created_at: i64,        // Creation timestamp
    pub last_activity: i64,     // Last deposit/withdrawal time
    pub is_locked: bool,        // Currently locked
    pub total_deposited: u64,   // Lifetime deposit sum
    pub total_withdrawn: u64,   // Lifetime withdrawal sum
}
```

### Escrow
Conditional two-party token release agreement.

```rust
pub struct Escrow {
    pub depositor: Pubkey,           // Party funding escrow
    pub beneficiary: Pubkey,         // Party receiving tokens
    pub token_mint: Pubkey,          // Token being escrowed
    pub escrow_token_account: Pubkey,// Account holding tokens
    pub amount: u64,                 // Escrowed amount
    pub release_time: i64,           // Unix timestamp for release
    pub created_at: i64,             // Creation timestamp
    pub is_released: bool,           // Released state
    pub is_cancelled: bool,          // Cancelled state
    pub bump: u8,                    // PDA bump seed
}
```

---

## Instructions

### 1. Initialize Treasury
Set up the main treasury account (admin-only operation).

**Accounts**:
- `authority` (signer) - Treasury admin
- `treasury` (write) - New treasury PDA
- `token_mint` (read) - Token to manage
- `vault_token_account` (write) - Token account for holding funds
- `token_program` (read) - SPL Token program
- `system_program` (read) - System program
- `rent` (read) - Rent sysvar

**Parameters**:
- `treasury_bump: u8` - PDA bump seed

### 2. Create Vault
Create a new time-locked vault for the caller.

**Accounts**:
- `user` (signer) - Vault owner
- `treasury` (write) - Parent treasury
- `user_vault` (write) - New vault PDA
- `system_program` (read) - System program

**Parameters**:
- `vault_name: String` - Name (max 32 chars)
- `lock_until: i64` - Unix timestamp (0 for no lock)

### 3. Deposit
Deposit tokens into a vault.

**Accounts**:
- `user` (signer) - Token owner
- `treasury` (write) - Tracks total deposits
- `user_vault` (write) - Destination vault
- `user_token_account` (read) - Source of tokens
- `vault_token_account` (write) - Treasury holding account
- `token_program` (read) - SPL Token program

**Parameters**:
- `amount: u64` - Amount to deposit

### 4. Withdraw
Withdraw tokens from a vault (if lock period has expired).

**Accounts**:
- `user` (signer) - Vault owner
- `treasury` (read) - For lock validation
- `user_vault` (write) - Source vault
- `user_token_account` (write) - Destination
- `vault_token_account` (write) - Treasury account
- `token_program` (read) - SPL Token program

**Parameters**:
- `amount: u64` - Amount to withdraw

### 5. Create Escrow
Create a conditional escrow agreement between two parties.

**Accounts**:
- `depositor` (signer) - Party funding escrow
- `beneficiary` (read) - Party receiving tokens
- `token_mint` (read) - Token type
- `escrow` (write) - New escrow PDA
- `escrow_token_account` (write) - Escrow holding account
- `depositor_token_account` (read) - Token source
- `token_program` (read) - SPL Token program
- `system_program` (read) - System program
- `rent` (read) - Rent sysvar

**Parameters**:
- `amount: u64` - Escrow amount
- `release_time: i64` - Unix timestamp for release
- `escrow_bump: u8` - PDA bump seed

### 6. Release Escrow
Release escrowed tokens to beneficiary (after release time passes).

**Accounts**:
- `caller` (signer) - Anyone can trigger release
- `escrow` (write) - Escrow being released
- `escrow_token_account` (write) - Token source
- `beneficiary_token_account` (write) - Destination
- `token_program` (read) - SPL Token program

**Parameters**: None

### 7. Cancel Escrow
Cancel escrow and return tokens to depositor (before release time only).

**Accounts**:
- `depositor` (signer) - Original funder
- `escrow` (write) - Escrow being cancelled
- `escrow_token_account` (write) - Token source
- `depositor_token_account` (write) - Destination
- `token_program` (read) - SPL Token program

**Parameters**: None

### 8. Extend Lock
Extend the lock period of an existing vault (can only increase lock time).

**Accounts**:
- `user` (signer) - Vault owner
- `user_vault` (write) - Vault to update

**Parameters**:
- `new_lock_until: i64` - New Unix timestamp (must be > current)

---

## Configuration

### Vault Constraints
```rust
pub const MAX_VAULT_NAME_LENGTH: usize = 32;
pub const MIN_LOCK_TIMESTAMP: i64 = 0;     // 0 = no lock
```

### Program Constants
```rust
pub const TREASURY_SEED: &[u8] = b"treasury";
pub const USER_VAULT_SEED: &[u8] = b"user_vault";
pub const ESCROW_SEED: &[u8] = b"escrow";
```

---

## Usage Examples

### Create and Deposit into a Vault
```rust
// 1. Create a time-locked vault (locks until timestamp)
let lock_until = Clock::get()?.unix_timestamp + (30 * 24 * 60 * 60); // 30 days
instruction::create_vault(
    ctx,
    "My Savings".to_string(),
    lock_until,
)?;

// 2. Deposit tokens into the vault
instruction::deposit(ctx, 1_000_000)?;

// Tokens locked until lock_until timestamp
```

### Create and Release Escrow
```rust
// 1. Depositor creates escrow agreement
let release_time = Clock::get()?.unix_timestamp + (7 * 24 * 60 * 60); // 7 days
instruction::create_escrow(
    ctx,
    500_000,    // amount
    release_time,
    escrow_bump,
)?;

// 2. After release time, anyone can trigger release
instruction::release_escrow(ctx)?;

// Beneficiary receives tokens
```

### Extend Vault Lock
```rust
let new_lock_time = Clock::get()?.unix_timestamp + (60 * 24 * 60 * 60); // 60 days
instruction::extend_lock(ctx, new_lock_time)?;

// Cannot reduce lock time - always increases
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
- Treasury initialization and state validation
- Vault creation with various lock times
- Deposit and withdrawal flows
- Lock period enforcement
- Escrow creation, release, and cancellation
- Error handling for invalid operations
- Boundary conditions (lock time validation)

---

## Security Considerations

### ✅ Time-Lock Enforcement
Solana's Clock sysvar ensures tamper-proof timestamp enforcement. Withdrawals are impossible before the lock period expires.

### ✅ Ownership Validation
Each vault and escrow has clear ownership. Only authorized accounts can modify state:
- Vault withdrawals require owner signature
- Escrow cancellation requires depositor signature
- Escrow release can be triggered by anyone (time-based)

### ✅ SPL Token Safety
All token transfers use CPI to the Token program. No direct token account manipulation.

### ✅ PDA Signer Pattern
Treasury and escrow accounts sign token transfers using their PDA seeds. Prevents unauthorized transfers.

### ✅ State Consistency
Vault and escrow state are updated atomically with token transfers. No partial state updates.

### ✅ Overflow Protection
All arithmetic checked with `.checked_add()` and `.checked_sub()` to prevent overflow/underflow.

---

## Performance Characteristics

- **Create Vault**: ~0.3 CUs - Account initialization only
- **Deposit**: ~1 CU - Token transfer + state update
- **Withdraw**: ~1-1.5 CUs - Time check + token transfer
- **Create Escrow**: ~1.5 CUs - Account creation + transfer
- **Release Escrow**: ~1 CU - Time check + transfer
- **Cancel Escrow**: ~1 CU - Time check + transfer
- **Extend Lock**: ~0.2 CUs - Arithmetic only

Total compute usage is minimal, well under 1.4M unit limits.

---

## Integration with Other Programs

### With tribewarez-pot-o
Miners lock up rewards in vaults to participate in governance. Time-locks enforce commitment periods.

### With tribewarez-staking
Staked tokens can be locked in vaults. Escrow used for conditional reward distributions to stakers.

### With tribewarez-swap
Swap program integrates with vault for conditional token swaps. Escrowed swaps prevent frontrunning.

---

## Deployment

See [DEPLOYMENT_GUIDE.md](../DEPLOYMENT_GUIDE.md) for detailed devnet/testnet deployment instructions.

Quick deployment from workspace root:
```bash
cd pot-o-contractz
anchor build -p tribewarez_vault
anchor deploy --provider.cluster devnet
```

---

## API Documentation

Complete API documentation is available on [docs.rs](https://docs.rs/tribewarez-vault).

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

- [tribewarez-pot-o](../tribewarez-pot-o) - PoT-O mining program
- [tribewarez-staking](../tribewarez-staking) - Staking pool management
- [tribewarez-swap](../tribewarez-swap) - AMM token swaps
- [pot-o-validator](../../pot-o-validator) - Off-chain validator daemon

## Resources

- [GitHub Repository](https://github.com/tribewarez/tribe)
- [Anchor Documentation](https://docs.anchor-lang.com)
- [Solana Program Library](https://spl.solana.com)
- [PoT-O Whitepaper](https://tribewarez.com/pot-o)
