# Quick Start Guide - Docker Build

## Prerequisites

- Docker installed and running
- Docker Compose installed
- ~10GB free disk space (for Docker images and build cache)

## Build Steps

### 1. Build the Docker Image (First Time - Takes 10-15 minutes)

```bash
cd programs
docker compose build anchor-build
```

This will:
- Download Ubuntu 22.04 base image
- Install Rust 1.70.0
- Install Solana CLI 1.16
- Install Anchor CLI 0.28.0
- Set up the build environment

**Note**: First build takes time. Subsequent builds are much faster due to Docker layer caching.

### 2. Build the Programs

```bash
# Option 1: Using the script
./build.sh

# Option 2: Using Make
make build

# Option 3: Direct docker-compose
docker compose run --rm anchor-build
```

### 3. Verify Build

Check that `.so` files were created:

```bash
ls -lh target/deploy/*.so
```

You should see:
- `tribewarez_staking.so`
- `tribewarez_vault.so`
- `tribewarez_swap.so`

### 4. Deploy to Devnet

```bash
# Make sure you have a Solana wallet configured
docker compose run --rm anchor-shell solana config set --url devnet

# Deploy
make deploy
```

## Troubleshooting

### Build Fails - "No such file or directory"
```bash
# Make sure you're in the programs directory
cd programs
```

### Build Fails - "Docker daemon not running"
```bash
# Start Docker
sudo systemctl start docker
# Or on macOS: open Docker Desktop
```

### Out of Disk Space
```bash
# Clean Docker
docker system prune -a
docker volume prune
```

### Slow Build
- First build is always slow (10-15 min)
- Subsequent builds use cache and are much faster (1-2 min)
- Make sure Docker has enough resources allocated

### Permission Denied
```bash
# Add your user to docker group (Linux)
sudo usermod -aG docker $USER
# Log out and back in
```

## Alternative: Use Pre-built Image

If you want to skip the build step, you can use a pre-built image (when available):

```bash
docker pull tribewarez/anchor-build:0.28
```

## Next Steps

After successful build:

1. **Deploy to devnet**: `make deploy`
2. **Update config**: Update `src/contracts/config.ts` with deployed program IDs
3. **Test**: Use the frontend hooks to interact with deployed programs
4. **Deploy to mainnet**: After thorough testing

## Common Commands

```bash
# Build
make build

# Deploy
make deploy

# Test
make test

# Clean
make clean

# Interactive shell
make shell

# Check versions
docker compose run --rm anchor-shell anchor --version
docker compose run --rm anchor-shell solana --version
```

## File Locations

- **Compiled programs**: `target/deploy/*.so`
- **IDL files**: `target/idl/*.json`
- **Keypairs**: `target/deploy/*-keypair.json`

## Support

- Check `DOCKER_BUILD.md` for detailed Docker documentation
- Check `README.md` for program documentation
- Check `DEPLOYMENT.md` for deployment details
