# tribewarez-swap

**Automated Market Maker (AMM)** - Constant product swap program for PTtC tokens on Solana with liquidity pools, LP tokens, slippage protection, and dynamic fee collection.

[![Crates.io](https://img.shields.io/crates/v/tribewarez-swap.svg)](https://crates.io/crates/tribewarez-swap)
[![docs.rs](https://docs.rs/tribewarez-swap/badge.svg)](https://docs.rs/tribewarez-swap)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Part of the [Tribewarez programs](../README.md) workspace and [PoT-O ecosystem](https://github.com/tribewarez).

---

## Overview

The `tribewarez-swap` program implements a Constant Product Automated Market Maker (AMM) for decentralized token swaps on Solana. It provides:

- **Liquidity pools** - Create pools for any token pair with constant product formula (x * y = k)
- **Token swaps** - Trade between token pairs with automatic price discovery
- **Liquidity provision** - Add/remove liquidity and earn swap fees as LP token holders
- **Protocol fees** - Collect 0.30% swap fee (0.25% to LPs, 0.05% to protocol)
- **Slippage protection** - Specify minimum output amounts for safe trading
- **Quote generation** - Query swap amounts and price impact before execution

This program works alongside PoT-O mining, staking, and vault programs to provide a complete trading infrastructure.

---

## Key Features

### 📊 Constant Product AMM (x * y = k)
Industry-standard AMM formula ensures:
- Automatic price discovery based on pool reserves
- Continuous liquidity without order books
- Predictable pricing mechanism

### 💧 Liquidity Pools
Create pools for any token pair with equal value provisioning:
- First LP receives `sqrt(amount_a * amount_b)` LP tokens
- Subsequent LPs receive proportional shares
- Pool reserves track total liquidity

### 🎟️ LP Tokens
Liquidity providers receive LP tokens representing their share:
- Burn LP tokens to withdraw proportional amounts
- Earn 0.25% of all swaps in your pool
- Transfer LP tokens to others

### 🛡️ Slippage Protection
Specify minimum output amounts for atomic transactions:
- Failed swaps revert with no partial execution
- Protection against flash loan attacks
- User-controlled execution

### 💰 Fee Structure
Automated fee collection and distribution:
- **Swap Fee**: 0.30% per transaction (30 basis points)
- **LP Fee**: 0.25% distributed to liquidity providers
- **Protocol Fee**: 0.05% collected for governance

### 📈 Price Impact Calculation
Get detailed pricing information before swaps:
- Price impact in basis points
- Slippage estimates
- Complete quote data

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────┐
│     User Initiates Swap or Liquidity Action              │
└──────────────────┬───────────────────────────────────────┘
                   │
        ┌──────────┴──────────────┬──────────────┐
        │                         │              │
        ▼                         ▼              ▼
  ┌────────────┐          ┌──────────────┐  ┌────────────┐
  │ Add        │          │ Swap (A→B)   │  │ Remove     │
  │ Liquidity  │          │              │  │ Liquidity  │
  │            │          │ - Check pool │  │            │
  │ - Transfer │          │ - Calculate  │  │ - Burn LPs │
  │   both     │          │   output     │  │ - Transfer │
  │   tokens   │          │ - Validate   │  │   both     │
  │ - Mint LPs │          │   slippage   │  │   tokens   │
  │ - Update   │          │ - Collect    │  │ - Update   │
  │   reserves │          │   fees       │  │   reserves │
  └────────────┘          └──────────────┘  └────────────┘
        │                         │              │
        │    ┌────────────────────┴──────────────┤
        │    │                                   │
        └────┴───────────────────────────────────┘
             │
             ▼
  ┌──────────────────────────────────────────┐
  │ Update Pool State (Reserves, Fees)       │
  │ Emit Events (LiquidityAdded, Swapped)    │
  └──────────────────────────────────────────┘
```

---

## Data Structures

### LiquidityPool
Central pool account managing all liquidity and swaps.

```rust
pub struct LiquidityPool {
    pub authority: Pubkey,           // Pool admin
    pub token_a_mint: Pubkey,        // First token in pair
    pub token_b_mint: Pubkey,        // Second token in pair
    pub token_a_vault: Pubkey,       // Pool reserves for token A
    pub token_b_vault: Pubkey,       // Pool reserves for token B
    pub lp_mint: Pubkey,             // LP token mint
    pub reserve_a: u64,              // Token A balance
    pub reserve_b: u64,              // Token B balance
    pub total_lp_supply: u64,        // Total LP tokens minted
    pub swap_fee_bps: u64,           // Swap fee in basis points
    pub protocol_fee_bps: u64,       // Protocol fee allocation
    pub collected_fees_a: u64,       // Accumulated protocol fees (A)
    pub collected_fees_b: u64,       // Accumulated protocol fees (B)
    pub bump: u8,                    // PDA bump seed
    pub is_active: bool,             // Accept trades
    pub created_at: i64,             // Creation timestamp
}
```

---

## Instructions

### 1. Initialize Pool
Create a new liquidity pool for a token pair.

**Accounts**:
- `authority` (signer) - Pool admin
- `pool` (write) - New pool PDA
- `token_a_mint`, `token_b_mint` (read) - Token mints
- `token_a_vault`, `token_b_vault` (write) - Pool reserve accounts
- `lp_mint` (write) - LP token mint
- `token_program`, `system_program`, `rent` (read) - System programs

**Parameters**:
- `pool_bump: u8` - PDA bump seed

### 2. Add Liquidity
Provide liquidity to a pool and receive LP tokens.

**Accounts**:
- `user` (signer) - Liquidity provider
- `pool` (write) - Target pool
- `user_token_a`, `user_token_b` (read) - Token sources
- `token_a_vault`, `token_b_vault` (write) - Pool reserves
- `user_lp_account` (write) - LP token destination
- `lp_mint` (write) - LP mint
- `token_program` (read) - SPL Token

**Parameters**:
- `amount_a: u64` - Token A deposit
- `amount_b: u64` - Token B deposit
- `min_lp_tokens: u64` - Slippage protection (minimum LP tokens)

**LP Token Calculation**:
- First LP: `sqrt(amount_a * amount_b)`
- Subsequent: `min(amount_a * total_lp / reserve_a, amount_b * total_lp / reserve_b)`

### 3. Remove Liquidity
Withdraw liquidity by burning LP tokens.

**Accounts**:
- `user` (signer) - LP holder
- `pool` (write) - Source pool
- `user_token_a`, `user_token_b` (write) - Token destinations
- `token_a_vault`, `token_b_vault` (write) - Pool reserves
- `user_lp_account` (write) - LP token source
- `lp_mint` (write) - LP mint
- `token_program` (read) - SPL Token

**Parameters**:
- `lp_amount: u64` - LP tokens to burn
- `min_amount_a: u64` - Slippage protection (minimum token A)
- `min_amount_b: u64` - Slippage protection (minimum token B)

**Token Return Calculation**:
- `amount_a = lp_amount * reserve_a / total_lp_supply`
- `amount_b = lp_amount * reserve_b / total_lp_supply`

### 4. Swap A for B
Trade token A for token B.

**Accounts**:
- `user` (signer) - Trader
- `pool` (write) - Swap pool
- `user_token_a` (write) - Input tokens
- `user_token_b` (write) - Output tokens
- `token_a_vault`, `token_b_vault` (write) - Pool reserves
- `token_program` (read) - SPL Token

**Parameters**:
- `amount_in: u64` - Token A input
- `min_amount_out: u64` - Slippage protection (minimum token B)

### 5. Swap B for A
Trade token B for token A (symmetric to Swap A for B).

**Accounts**: Same as Swap A for B (tokens swapped)

**Parameters**: Same as Swap A for B

### 6. Get Swap Quote
Query swap output and price impact (read-only).

**Accounts**:
- `pool` (read) - Quote pool

**Parameters**:
- `amount_in: u64` - Input amount
- `is_a_to_b: bool` - Direction (true = A→B, false = B→A)

**Emits**: SwapQuote event with amount_out, fee, price_impact

### 7. Withdraw Fees
Admin function to collect protocol fees.

**Accounts**:
- `authority` (signer) - Pool admin
- `pool` (write) - Fee pool
- `token_a_vault`, `token_b_vault` (write) - Fee sources
- `fee_receiver_a`, `fee_receiver_b` (write) - Fee destinations
- `token_program` (read) - SPL Token

**Parameters**: None

---

## Configuration

### Fee Structure (Basis Points)
```rust
pub const SWAP_FEE_BPS: u64 = 30;        // 0.30% swap fee
pub const PROTOCOL_FEE_BPS: u64 = 5;     // 0.05% to protocol
pub const LP_FEE_BPS: u64 = 25;          // 0.25% to liquidity providers
```

### Constant Product Formula
```
output = (reserve_out * amount_in * (10000 - fee)) / (reserve_in * 10000 + amount_in * (10000 - fee))
```

### Price Impact
```
price_impact_bps = (amount_in * 10000) / reserve_in
```

---

## Usage Examples

### Create a Pool
```rust
// Create pool for trading Token A ↔ Token B
instruction::initialize_pool(ctx, pool_bump)?;

// Pool now accepts liquidity and swaps
```

### Add Liquidity
```rust
// Add 100 Token A + 200 Token B
instruction::add_liquidity(
    ctx,
    100_000_000,  // amount_a (with decimals)
    200_000_000,  // amount_b
    1_000_000,    // min_lp_tokens (slippage: accept ≥1M LP)
)?;

// Receive LP tokens representing pool share
```

### Execute a Swap
```rust
// Swap 10 Token A for Token B
instruction::swap_a_for_b(
    ctx,
    10_000_000,   // amount_in
    9_000_000,    // min_amount_out (slippage: accept ≥9M Token B)
)?;

// Fee: 0.30% (30K tokens) retained by pool
```

### Get Quote Before Swap
```rust
// Check output before committing
instruction::get_swap_quote(ctx, 10_000_000, true)?;

// SwapQuote event emitted with:
// - amount_out: calculated output
// - fee: expected fee
// - price_impact: expected slippage
```

### Remove Liquidity
```rust
// Burn 1M LP tokens
instruction::remove_liquidity(
    ctx,
    1_000_000,    // lp_amount
    90_000_000,   // min_amount_a (slippage)
    180_000_000,  // min_amount_b (slippage)
)?;

// Receive proportional amounts of both tokens
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
- Pool initialization and state validation
- Add/remove liquidity flows
- Swap execution with fee calculation
- Slippage protection validation
- LP token minting/burning
- Price impact calculations
- Boundary conditions (zero reserves, etc.)

---

## Security Considerations

### ✅ Constant Product Enforcement
The formula `x * y = k` is maintained for all operations. Swaps cannot deplete reserves below this invariant.

### ✅ Slippage Protection
Users specify minimum acceptable outputs. Transactions revert if price moves unfavorably, preventing MEV exploitation.

### ✅ PDA Signers
Pool PDAs sign all outgoing transfers. Only the program can move pool assets.

### ✅ Fee Safety
Fees collected separately and can only be withdrawn by admin via explicit instruction.

### ✅ Overflow Prevention
All arithmetic uses `.checked_*` operations. Math errors cause transaction revert.

### ✅ Token Program Integration
All transfers via CPI to Token program. No direct account manipulation.

### ✅ LP Token Mechanics
LP tokens are standard SPL tokens. Can be transferred, but burning always returns proportional assets.

---

## Performance Characteristics

- **Initialize Pool**: ~1 CU - Account creation
- **Add Liquidity**: ~2-2.5 CUs - Two transfers + LP minting
- **Remove Liquidity**: ~2-2.5 CUs - LP burning + two transfers
- **Swap**: ~2-2.5 CUs - Input transfer + output transfer + fee tracking
- **Get Quote**: ~0.5 CU - Read-only calculations
- **Withdraw Fees**: ~1-2 CUs - Up to two fee transfers

All operations well under 1.4M per-transaction limit.

---

## Integration with Other Programs

### With tribewarez-pot-o
Miners trade rewards on liquidity pools. Mining rewards compatible with all pool token pairs.

### With tribewarez-staking
Staking rewards can be swapped for other tokens. LP tokens from staking pools are tradeable.

### With tribewarez-vault
Escrowed tokens can be swapped atomically. Time-locked vaults prevent frontrunning on swaps.

---

## Deployment

See [DEPLOYMENT_GUIDE.md](../DEPLOYMENT_GUIDE.md) for detailed devnet/testnet deployment instructions.

Quick deployment from workspace root:
```bash
cd pot-o-contractz
anchor build -p tribewarez_swap
anchor deploy --provider.cluster devnet
```

---

## API Documentation

Complete API documentation is available on [docs.rs](https://docs.rs/tribewarez-swap).

For trait-based service integration, see [SERVICE_API_REFERENCE.md](../SERVICE_API_REFERENCE.md).

---

## Contributing

Contributions welcome! Please follow:
- Conventional commit messages (feat:, fix:, docs:, test:)
- 80%+ test coverage for new code
- Clear documentation for public APIs
- Security audit for mathematical correctness

---

## License

MIT - See [LICENSE](LICENSE) for details.

---

## Related Programs

- [tribewarez-pot-o](../tribewarez-pot-o) - PoT-O mining program
- [tribewarez-staking](../tribewarez-staking) - Staking pool management
- [tribewarez-vault](../tribewarez-vault) - Escrow and treasury
- [pot-o-validator](../../pot-o-validator) - Off-chain validator daemon

## Resources

- [GitHub Repository](https://github.com/tribewarez/tribe)
- [Anchor Documentation](https://docs.anchor-lang.com)
- [Solana Program Library](https://spl.solana.com)
- [PoT-O Whitepaper](https://tribewarez.com/pot-o)
