# Tribewarez Smart Contract Deployment Guide

Complete guide for deploying all four tribewarez programs to Solana devnet, testnet, and production networks.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Environment Setup](#environment-setup)
3. [Quick Start (5-10 minutes)](#quick-start)
4. [Individual Program Deployment](#individual-program-deployment)
5. [Verification Procedures](#verification-procedures)
6. [Program ID Matrix](#program-id-matrix)
7. [Testnet PDA Configuration](#testnet-pda-configuration)
8. [Network Configuration](#network-configuration)
9. [Key Management](#key-management)
10. [Troubleshooting](#troubleshooting)
11. [Deployment Checklist](#deployment-checklist)

---

## Prerequisites

### Required Software

- **Rust** (1.70+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Anchor CLI** (0.28+): `npm install -g @coral-xyz/anchor-cli`
- **Solana CLI** (1.16+): `sh -c "$(curl -sSfL https://release.solana.com/stable/install)"`
- **Node.js** (16+): Required for Anchor CLI
- **Git**: For repository access

### Verify Installation

```bash
rustc --version          # Rust compiler
anchor --version         # Anchor framework
solana --version         # Solana CLI
npm --version            # Node.js
```

### System Requirements

- 4GB RAM (8GB+ recommended)
- 10GB free disk space (for build artifacts)
- Stable internet connection (for network communication)

---

## Environment Setup

### 1. Configure Solana CLI

Set default network and keypair:

```bash
# For Devnet (default)
solana config set --url https://api.devnet.solana.com
solana config set --keypair ~/.config/solana/devnet.json

# For Testnet
solana config set --url https://api.testnet.solana.com
solana config set --keypair ~/.config/solana/testnet.json
```

### 2. Create/Import Keypair

Generate new keypair:
```bash
solana-keygen new --outfile ~/.config/solana/devnet.json
```

Or use existing:
```bash
solana-keygen recover --keypair ~/.config/solana/devnet.json
```

### 3. Fund Your Keypair

For devnet (faucet):
```bash
solana airdrop 2 --url devnet
```

For testnet:
```bash
solana airdrop 2 --url testnet
```

Check balance:
```bash
solana balance --url devnet
```

### 4. Create Anchor Configuration

Create `Anchor.toml` at workspace root with:

```toml
[provider]
cluster = "Devnet"
wallet = "~/.config/solana/devnet.json"

[programs.devnet]
tribewarez_pot_o = "HmWGA3JAF6basxGCvvGNHAdTBE3qCPhJCeFJAd7r5ra9"
tribewarez_staking = "9keqqbmhSgLDNcnJV3g3ZZLB8NJHHU4X1D6p9p3cZBoC"
tribewarez_vault = "HmWGA3JAF6basxGCvvGNHAdTBE3qCPhJCeFJAd7r5ra9"
tribewarez_swap = "GPGGnKwnvKseSxzPukrNvch1CwYhifTqgj2RdW1P26H3"

[programs.testnet]
tribewarez_pot_o = "[INSERT TESTNET PID]"
tribewarez_staking = "[INSERT TESTNET PID]"
tribewarez_vault = "[INSERT TESTNET PID]"
tribewarez_swap = "[INSERT TESTNET PID]"

[scripts]
test = "mocha --require ts-node/register tests/**/*.test.ts"
```

---

## Quick Start

Deploy all programs to devnet in 5-10 minutes:

### Step 1: Clone and Navigate

```bash
cd /path/to/pot-
cd pot-o-contractz
```

### Step 2: Install Dependencies

```bash
npm install
```

### Step 3: Build All Programs

```bash
anchor build
```

This produces:
- `target/idl/` - TypeScript IDL files
- `target/deploy/` - Program bytecode

### Step 4: Deploy All Programs

```bash
anchor deploy --provider.cluster devnet
```

Output shows:
- Program ID (written to cluster)
- Transaction signature
- RPC confirmation

### Step 5: Verify Deployment

```bash
# Check each program is on-chain
solana program show HmWGA3JAF6basxGCvvGNHAdTBE3qCPhJCeFJAd7r5ra9 --url devnet
solana program show 9keqqbmhSgLDNcnJV3g3ZZLB8NJHHU4X1D6p9p3cZBoC --url devnet
```

Success: Program displays "Owner: BPFLoaderUpgradeable"

---

## Individual Program Deployment

### tribewarez-pot-o (PoT-O Mining)

Deploy mining program:

```bash
cd pot-o-contractz

# Build only this program
anchor build -p tribewarez_pot_o

# Deploy
solana program deploy \
  target/deploy/tribewarez_pot_o.so \
  --program-id ~/.config/solana/pot-o-keypair.json \
  --url devnet

# Verify
solana program show HmWGA3JAF6basxGCvvGNHAdTBE3qCPhJCeFJAd7r5ra9 --url devnet
```

**Program ID**: `HmWGA3JAF6basxGCvvGNHAdTBE3qCPhJCeFJAd7r5ra9`

### tribewarez-staking (Staking Pool)

```bash
anchor build -p tribewarez_staking

solana program deploy \
  target/deploy/tribewarez_staking.so \
  --program-id ~/.config/solana/staking-keypair.json \
  --url devnet
```

**Program ID**: `9keqqbmhSgLDNcnJV3g3ZZLB8NJHHU4X1D6p9p3cZBoC`

### tribewarez-vault (Escrow & Treasury)

```bash
anchor build -p tribewarez_vault

solana program deploy \
  target/deploy/tribewarez_vault.so \
  --program-id ~/.config/solana/vault-keypair.json \
  --url devnet
```

**Program ID**: `HmWGA3JAF6basxGCvvGNHAdTBE3qCPhJCeFJAd7r5ra9`

### tribewarez-swap (AMM)

```bash
anchor build -p tribewarez_swap

solana program deploy \
  target/deploy/tribewarez_swap.so \
  --program-id ~/.config/solana/swap-keypair.json \
  --url devnet
```

**Program ID**: `GPGGnKwnvKseSxzPukrNvch1CwYhifTqgj2RdW1P26H3`

---

## Verification Procedures

### 1. Verify Program Deployed

```bash
# Program should show "Status: Ok" and BPFLoaderUpgradeable owner
solana program show <PROGRAM_ID> --url <NETWORK>
```

### 2. Verify Program Size

Check bytecode size (must be < 12.8MB for on-chain limits):

```bash
ls -lh target/deploy/tribewarez_*.so
```

### 3. Verify Program Upgrade Authority

```bash
# Get program info including upgrade authority
solana program show <PROGRAM_ID> --url <NETWORK> --output json
```

### 4. Run Integration Tests

```bash
# Test against deployed program
anchor test --provider.cluster devnet
```

Tests should verify:
- Program initialization
- Account creation
- Instruction execution
- Event emission

### 5. Verify Account Derivations

Test PDA derivation matches on-chain:

```bash
# Check treasury PDA for vault program
solana account \
  "vault:treasury:PTTmf8NMUekRFuVJPSKHLrkyMTWWkvKVUkhMVGGKFwY" \
  --url devnet
```

---

## Program ID Matrix

### Devnet (Active)

| Program | ID | Deployed | Authority |
|---------|----|-----------|-|
| PoT-O | `HmWGA3JAF6basxGCvvGNHAdTBE3qCPhJCeFJAd7r5ra9` | ✅ | Devnet Authority |
| Staking | `9keqqbmhSgLDNcnJV3g3ZZLB8NJHHU4X1D6p9p3cZBoC` | ✅ | Devnet Authority |
| Vault | `HmWGA3JAF6basxGCvvGNHAdTBE3qCPhJCeFJAd7r5ra9` | ✅ | Devnet Authority |
| Swap | `GPGGnKwnvKseSxzPukrNvch1CwYhifTqgj2RdW1P26H3` | ✅ | Devnet Authority |

### Testnet (To Deploy)

| Program | ID | Deployed | Authority |
|---------|----|-----------|-|
| PoT-O | TBD | ⏳ | Testnet Authority |
| Staking | TBD | ⏳ | Testnet Authority |
| Vault | TBD | ⏳ | Testnet Authority |
| Swap | TBD | ⏳ | Testnet Authority |

### Mainnet (Reserved)

| Program | ID | Deployed | Authority |
|---------|----|-----------|-|
| PoT-O | TBD | ⏳ | DAO Governance |
| Staking | TBD | ⏳ | DAO Governance |
| Vault | TBD | ⏳ | DAO Governance |
| Swap | TBD | ⏳ | DAO Governance |

---

## Testnet PDA Configuration

For testnet/mainnet deployments, upgrade these PDAs to be upgradeable:

### Vault Program Testnet Setup

```bash
# Initialize vault on testnet (vs hardcoded devnet)
# Ensure all PDAs use correct program ID seeds

# Treasury PDA:
Seed: [b"treasury", token_mint.as_ref()]

# User Vault PDA:
Seed: [b"user_vault", treasury.key(), user.key()]

# Escrow PDA:
Seed: [b"escrow", depositor.key(), beneficiary.key()]
```

### Swap Program Testnet Setup

```bash
# Pool PDA for testnet:
Seed: [b"pool", token_a_mint.as_ref(), token_b_mint.as_ref()]

# All pool vaults and LP mints tied to pool address
```

### Staking Program Testnet Setup

```bash
# Pool PDA:
Seed: [b"pool", authority.key()]

# Stake account PDA:
Seed: [b"stake", pool.key(), staker.key()]
```

### PoT-O Program Testnet Setup

```bash
# Miner PDA:
Seed: [b"miner", authority.key()]

# Difficulty state PDA:
Seed: [b"difficulty"]
```

---

## Network Configuration

### RPC Endpoints

```bash
# Devnet
https://api.devnet.solana.com
wss://api.devnet.solana.com

# Testnet
https://api.testnet.solana.com
wss://api.testnet.solana.com

# Mainnet Beta
https://api.mainnet-beta.solana.com
wss://api.mainnet-beta.solana.com

# Local (for testing)
http://localhost:8899
ws://localhost:8900
```

### Network Parameters

```toml
[provider]
cluster = "Devnet"          # or "Testnet", "Mainnet"
wallet = "~/.config/solana/devnet.json"
commitment = "finalized"    # "finalized", "confirmed", "processed"
```

### Fees

- **Devnet**: Minimal/free airdrops
- **Testnet**: Free airdrops + some rate limiting
- **Mainnet**: Full SOL fees (~0.00025 SOL per transaction)

---

## Key Management

### ⚠️ Security Best Practices

1. **Never commit keypairs** to git (add to `.gitignore`)
2. **Use hardware wallets** for mainnet
3. **Rotate keys regularly** on testnet
4. **Keep upgrade authorities separate** from deploy authority
5. **Use multi-sig for mainnet** governance

### Keypair Storage

```bash
# Secure storage locations
~/.config/solana/devnet.json      # Devnet (development)
~/.config/solana/testnet.json     # Testnet (staging)
/secure/path/mainnet.json         # Mainnet (hardware wallet recommended)
```

### Upgrade Authority Management

```bash
# Set non-upgradeable (irreversible!)
solana program set-upgrade-authority <PROGRAM_ID> --new-upgrade-authority none

# Transfer authority
solana program set-upgrade-authority <PROGRAM_ID> \
  --new-upgrade-authority <NEW_AUTHORITY_ADDRESS>
```

---

## Troubleshooting

### Build Failures

**Error**: `anchor build` fails with LLVM errors

**Solution**:
```bash
# Clean and rebuild
anchor clean
cargo clean
anchor build
```

### Deployment Failures

**Error**: `Program failed to deploy (account too large)`

**Solution**:
```bash
# Check bytecode size
ls -lh target/deploy/tribewarez_*.so

# If > 12.8MB, optimize Cargo.toml:
# Add to Cargo.toml:
[profile.release]
lto = true
codegen-units = 1
opt-level = "z"
strip = true
```

### RPC Connection Errors

**Error**: `RPC request failed: connection refused`

**Solution**:
```bash
# Verify RPC is online
solana ping --url devnet

# Use fallback RPC
anchor deploy --provider.cluster devnet \
  --provider.opts.rpc https://api.devnet.solana.com
```

### Insufficient Lamports

**Error**: `Transaction failed: insufficient funds`

**Solution**:
```bash
# Check balance
solana balance --url devnet

# Airdrop more SOL
solana airdrop 5 --url devnet
solana airdrop 5 --url testnet
```

### PDA Mismatch

**Error**: `Program error: custom program error: 0x0`

**Solution**:
```bash
# Verify PDAs match on-chain:
# Check Anchor.toml program IDs
# Verify seeds match in Rust code
# Ensure bump derivation is correct

# Use CLI to find PDA:
solana address-lookup-table create
```

### Anchor Version Mismatch

**Error**: `Version conflict: Anchor CLI vs package.json`

**Solution**:
```bash
# Update all to same version
npm install -g @coral-xyz/anchor-cli@0.28.0

# Update package.json
npm install @coral-xyz/anchor@0.28.0

# Verify
anchor --version
```

---

## Deployment Checklist

Use this checklist before deploying to each network:

### Pre-Deployment

- [ ] All tests pass: `anchor test --provider.cluster devnet`
- [ ] Code reviewed and audited
- [ ] Changelog updated with new features
- [ ] Version bumped in Cargo.toml (following semver)
- [ ] IDL generated and matches program: `anchor build`
- [ ] No hardcoded network-specific values
- [ ] All TODOs resolved or documented

### Devnet Deployment

- [ ] Keypair funded with testnet SOL
- [ ] Solana CLI set to devnet: `solana config set --url devnet`
- [ ] Anchor configured for devnet in Anchor.toml
- [ ] Build successful: `anchor build`
- [ ] Deployment succeeds: `anchor deploy --provider.cluster devnet`
- [ ] Program verified on-chain: `solana program show <PID>`
- [ ] Integration tests pass against deployed program
- [ ] PDAs derive correctly on-chain

### Testnet Deployment

- [ ] All devnet tests passing
- [ ] Program bytecode unchanged from devnet (for consistency)
- [ ] Testnet keypair configured: `solana config set --keypair testnet.json`
- [ ] Testnet RPC verified: `solana ping --url testnet`
- [ ] Sufficient testnet SOL (minimum 2 SOL)
- [ ] Deployment succeeds: `anchor deploy --provider.cluster testnet`
- [ ] Same verification procedures as devnet
- [ ] Testnet PDAs documented in Program ID Matrix

### Mainnet Deployment (DAO Governance)

- [ ] All testnet testing complete (minimum 2 weeks)
- [ ] Security audit completed by external firm
- [ ] Upgrade authority set to DAO multi-sig
- [ ] Mainnet keypair is hardware wallet
- [ ] Liquidity pools seeded with initial capital
- [ ] Oracle prices verified and operational
- [ ] Governance parameters approved by DAO
- [ ] Deployment uses formal governance proposal

### Post-Deployment

- [ ] Announce deployment with program IDs
- [ ] Monitor logs for errors: `solana logs <PID> --url <NETWORK>`
- [ ] Verify all accounts created correctly
- [ ] Run smoke tests on testnet/mainnet
- [ ] Document any network-specific configurations
- [ ] Update documentation with deployed addresses
- [ ] Schedule security review for mainnet

---

## Support

For deployment issues:

1. Check **Troubleshooting** section above
2. Review **Anchor Documentation**: https://docs.anchor-lang.com
3. Check **Solana Program Library**: https://spl.solana.com
4. Open issue on GitHub with logs

---

## Related Documentation

- [tribewarez-pot-o README](tribewarez-pot-o/README.md) - PoT-O mining program
- [tribewarez-staking README](tribewarez-staking/README.md) - Staking pools
- [tribewarez-vault README](tribewarez-vault/README.md) - Escrow & treasury
- [tribewarez-swap README](tribewarez-swap/README.md) - AMM swaps
- [SERVICE_API_REFERENCE.md](SERVICE_API_REFERENCE.md) - Service trait documentation