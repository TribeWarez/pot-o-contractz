# Network/SSL Issues - Workaround Guide

If you're experiencing SSL connection errors when building the Docker image, here are several workarounds:

## Option 1: Use Alternative Dockerfile (No Solana in Build)

Use `Dockerfile.alternative` which doesn't install Solana during build:

```bash
# Build without Solana
docker build -f Dockerfile.alternative -t tribewarez-anchor-build .

# Install Solana manually inside container
docker run -it tribewarez-anchor-build bash
# Inside container:
bash INSTALL_SOLANA_MANUAL.sh
```

## Option 2: Use Host System Solana

If Solana is already installed on your host system:

```bash
# Build Docker image without Solana
docker build -f Dockerfile.alternative -t tribewarez-anchor-build .

# Run with host Solana mounted
docker run -it \
  -v $(which solana):/usr/local/bin/solana:ro \
  -v $(pwd):/workspace \
  tribewarez-anchor-build bash
```

## Option 3: Pre-download Solana Installer

Download the installer on your host first:

```bash
# Download installer
curl -L https://release.solana.com/stable/install -o solana-install.sh

# Copy into Docker build context
cp solana-install.sh programs/

# Modify Dockerfile to use local file:
# COPY solana-install.sh /tmp/
# RUN sh /tmp/solana-install.sh
```

## Option 4: Use Docker Build Args for Proxy

If behind a proxy:

```bash
docker build \
  --build-arg HTTP_PROXY=http://proxy:port \
  --build-arg HTTPS_PROXY=http://proxy:port \
  -t tribewarez-anchor-build .
```

## Option 5: Build Without Solana (Anchor Can Build Without It)

Anchor can build programs without Solana CLI. You only need Solana for deployment:

```bash
# Build programs (works without Solana)
docker build -f Dockerfile.alternative -t tribewarez-anchor-build .
docker run -v $(pwd):/workspace tribewarez-anchor-build \
  sh -c "cd programs && anchor build"

# Deploy using host Solana
solana program deploy target/deploy/tribewarez_staking.so
```

## Option 6: Use Pre-built Docker Image

If available, use a pre-built image:

```bash
docker pull tribewarez/anchor-build:latest
```

## Quick Fix: Skip Solana Installation

Modify the Dockerfile to skip Solana installation:

```dockerfile
# Comment out Solana installation
# RUN sh -c "$(curl -sSfL https://release.solana.com/stable/install)" ...

# Install Solana manually after container starts
```

Then install Solana when you run the container:

```bash
docker run -it your-image bash
# Inside: curl -sSfL https://release.solana.com/stable/install | sh
```

## Recommended Approach

For now, use **Option 5** - build without Solana, deploy with host Solana:

```bash
# 1. Build Docker image (without Solana)
docker build -f Dockerfile.alternative -t tribewarez-build .

# 2. Build programs
docker run -v $(pwd):/workspace tribewarez-build \
  sh -c "cd /workspace/programs && anchor build"

# 3. Deploy using your host Solana
cd programs
solana program deploy target/deploy/tribewarez_staking.so --program-id target/deploy/tribewarez_staking-keypair.json
```

This approach:
- ✅ Avoids SSL issues
- ✅ Uses your existing Solana installation
- ✅ Still provides consistent Rust/Anchor environment
- ✅ Works for both building and deploying
