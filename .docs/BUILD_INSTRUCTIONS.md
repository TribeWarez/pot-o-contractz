# Build Instructions - Network Issue Workaround

## Current Issue

The Docker build is encountering SSL connection errors when trying to download Solana. This is a network/SSL issue, not a code issue.

## ✅ Solution: Build Without Solana in Docker

**Good News**: Anchor can build programs **without** Solana CLI installed. You only need Solana for deployment, which can use your host system's Solana installation.

## Quick Build Steps

### Step 1: Build Docker Image (Without Solana)

```bash
cd programs

# Build using alternative Dockerfile (no Solana installation)
docker build -f Dockerfile.alternative -t tribewarez-build .
```

This installs:
- ✅ Rust 1.70.0
- ✅ Anchor CLI 0.28.0
- ❌ Solana CLI (skipped - use host system)

**Time**: ~5-10 minutes (installing Rust and Anchor)

### Step 2: Build Programs

```bash
# Build all Anchor programs
docker run --rm -v $(pwd):/workspace tribewarez-build \
  sh -c "cd /workspace/programs && anchor build"
```

**Time**: ~2-5 minutes (compiling programs)

### Step 3: Verify Build

```bash
# Check compiled programs exist
ls -lh target/deploy/*.so
```

You should see:
- `tribewarez_staking.so`
- `tribewarez_vault.so`  
- `tribewarez_swap.so`

### Step 4: Deploy Using Host Solana

Since Solana is already installed on your system, use it for deployment:

```bash
# Set to devnet
solana config set --url devnet

# Deploy programs
solana program deploy target/deploy/tribewarez_staking.so \
  --program-id target/deploy/tribewarez_staking-keypair.json \
  --keypair target/deploy/tribewarez_staking-keypair.json

solana program deploy target/deploy/tribewarez_vault.so \
  --program-id target/deploy/tribewarez_vault-keypair.json \
  --keypair target/deploy/tribewarez_vault-keypair.json

solana program deploy target/deploy/tribewarez_swap.so \
  --program-id target/deploy/tribewarez_swap-keypair.json \
  --keypair target/deploy/tribewarez_swap-keypair.json
```

## Alternative: Use Makefile (Auto-Fallback)

The Makefile has been updated to automatically fall back to the alternative method:

```bash
make build
```

It will:
1. Try docker compose build (with Solana)
2. If that fails, use Dockerfile.alternative (without Solana)
3. Build programs using Docker
4. You deploy using host Solana

## Why This Works

- **Building**: Anchor only needs Rust and Anchor CLI to compile programs
- **Deploying**: Solana CLI is only needed to deploy `.so` files to the network
- **Your System**: Already has Solana CLI installed (we verified earlier)

## Complete Workflow

```bash
# 1. Build Docker image (one time)
cd programs
docker build -f Dockerfile.alternative -t tribewarez-build .

# 2. Build programs (whenever you change code)
docker run --rm -v $(pwd):/workspace tribewarez-build \
  sh -c "cd /workspace/programs && anchor build"

# 3. Deploy (using your host Solana)
solana program deploy target/deploy/tribewarez_staking.so \
  --program-id target/deploy/tribewarez_staking-keypair.json
```

## Troubleshooting

### Docker Build Still Fails

```bash
# Clean Docker cache
docker system prune -a

# Rebuild
docker build -f Dockerfile.alternative -t tribewarez-build .
```

### Programs Don't Build

```bash
# Check Anchor version in container
docker run --rm tribewarez-build anchor --version

# Check Rust version
docker run --rm tribewarez-build rustc --version
```

### Deployment Fails

```bash
# Make sure Solana is configured
solana config get

# Check you have SOL for fees
solana balance

# Get some SOL from devnet faucet if needed
solana airdrop 2
```

## Summary

✅ **Docker Environment**: Ready (uses Dockerfile.alternative)
✅ **Build Process**: Works without Solana in Docker
✅ **Deployment**: Use host system Solana
✅ **All Programs**: Ready to build and deploy

The SSL issue only affects Solana installation in Docker, which we can work around by using your host system's Solana for deployment.
