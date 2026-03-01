# Tribewarez DeFi Smart Contracts - Deployment Guide

## Current Status

✅ **Smart Contracts Implemented:**
- ✅ Staking Program (`tribewarez-staking`) - Complete Rust implementation
- ✅ Vault Program (`tribewarez-vault`) - Complete Rust implementation  
- ✅ Swap AMM Program (`tribewarez-swap`) - Complete Rust implementation

✅ **Frontend Integration:**
- ✅ TypeScript SDK (`src/contracts/`)
- ✅ React Hooks (`src/hooks/use-staking.ts`, `src/hooks/use-swap.ts`)
- ✅ UI Components (`StakingPanel.tsx`, `SwapPanel.tsx`, `VaultPanel.tsx`)

⚠️ **Build Issue:**
There's a dependency compatibility issue between:
- Solana BPF toolchain (uses Rust 1.77.0 internally)
- Modern crates requiring Rust edition 2024 (Rust 1.87+)

## Program IDs Generated

The following program IDs have been generated and configured:

- **Staking Program**: `Go2BZRhNLoaVni3QunrKPAXYdHtwZtTXuVspxpdAeDS8`
- **Vault Program**: `HmWGA3JAF6basxGCvvGNHAdTBE3qCPhJCeFJAd7r5ra9`
- **Swap Program**: `GPGGnKwnvKseSxzPukrNvch1CwYhifTqgj2RdW1P26H3`

Keypairs are stored in:
- `target/deploy/tribewarez_staking-keypair.json`
- `target/deploy/tribewarez_vault-keypair.json`
- `target/deploy/tribewarez_swap-keypair.json`

## Solutions

### Option 1: Use Anchor 0.28 (Recommended)

Anchor 0.28 is more compatible with Solana 1.18:

```bash
cargo install anchor-cli --version 0.28.0
```

Then update `programs/Cargo.toml`:
```toml
[workspace.dependencies]
anchor-lang = "0.28.0"
spl-token = "3.0"
```

### Option 2: Use Solana 1.16

Install an older Solana version:
```bash
sh -c "$(curl -sSfL https://release.anza.xyz/v1.16.28/install)"
```

### Option 3: Build on a Different Machine

The contracts are ready to build. The issue is environment-specific. Try:
- macOS with Homebrew-installed Solana
- A CI/CD environment with proper toolchain versions
- Docker container with pre-configured Solana/Anchor versions

### Option 4: Manual Compilation

If you have access to a working Solana development environment:

```bash
cd programs
anchor build
anchor deploy
```

## Program Features

### Staking Program
- Stake/unstake PTtC tokens
- Earn rewards based on time staked
- Configurable lock periods
- Emergency unstake (forfeit rewards)
- Admin pool management

### Vault Program  
- Personal token vaults
- Time-locked savings
- Two-party escrow
- Deposit/withdrawal tracking

### Swap Program
- Constant product AMM (x * y = k)
- Add/remove liquidity
- Token swaps with slippage protection
- 0.30% swap fee (0.25% LP, 0.05% protocol)

## Frontend Usage

The frontend is ready to use once programs are deployed:

```typescript
import { useStaking } from './hooks/use-staking';
import { useSwap } from './hooks/use-swap';

// In your component
const { stake, unstake, claimRewards, stakedAmount, pendingRewards } = useStaking();
const { swap, getQuote, amountOut } = useSwap();
```

## Next Steps

1. **Resolve build environment** using one of the options above
2. **Build programs**: `anchor build`
3. **Deploy to devnet**: `anchor deploy --provider.cluster devnet`
4. **Update program IDs** in `src/contracts/config.ts` with deployed addresses
5. **Initialize pools** using admin scripts or frontend
6. **Test on devnet** before mainnet deployment

## Configuration Files

- `programs/Anchor.toml` - Anchor workspace config
- `programs/Cargo.toml` - Rust workspace dependencies
- `src/contracts/config.ts` - Frontend program IDs and token configs

## Support

For deployment assistance:
- Check Anchor docs: https://www.anchor-lang.com/docs
- Solana docs: https://docs.solana.com/
- Tribewarez: https://tribewarez.com
