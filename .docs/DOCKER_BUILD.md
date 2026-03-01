# Docker Build Environment for Tribewarez DeFi Contracts

This Docker setup provides a consistent, reproducible build environment for Solana smart contracts using compatible versions of Anchor and Solana.

## Quick Start

### Prerequisites
- Docker installed
- Docker Compose installed

### Build Programs

```bash
# Option 1: Using the build script
chmod +x build.sh
./build.sh

# Option 2: Using Make
make build

# Option 3: Using Docker Compose directly
docker compose build anchor-build
docker compose run --rm anchor-build
```

### Deploy Programs

```bash
# Deploy to devnet
make deploy

# Or using docker-compose
docker compose run --rm anchor-shell anchor deploy --provider.cluster devnet
```

### Interactive Shell

```bash
# Open an interactive shell in the build environment
make shell

# Or using docker-compose
docker compose run --rm anchor-shell
```

## Environment Details

The Docker image includes:

- **Ubuntu 22.04** - Base OS
- **Rust 1.70.0** - Compatible Rust version
- **Solana CLI 1.16.28** - Stable Solana version
- **Anchor CLI 0.28.0** - Compatible Anchor version

## Volume Mounts

- `./` → `/workspace` - Your project files
- `cargo-cache` - Cargo registry cache (persistent)
- `solana-cache` - Solana installation cache (persistent)

## Common Commands

### Build
```bash
make build
```

### Deploy to Devnet
```bash
make deploy
```

### Run Tests
```bash
make test
```

### Clean Build Artifacts
```bash
make clean
```

### Check Versions
```bash
docker compose run --rm anchor-shell anchor --version
docker compose run --rm anchor-shell solana --version
```

### Generate IDL
```bash
docker compose run --rm anchor-shell anchor build
# IDL files will be in target/idl/
```

## Troubleshooting

### Build Fails with Lock File Error
```bash
# Clean and rebuild
make clean
make build
```

### Permission Issues
```bash
# Fix file permissions
sudo chown -R $USER:$USER .
```

### Docker Image Build Fails
```bash
# Rebuild without cache
docker compose build --no-cache anchor-build
```

### Out of Disk Space
```bash
# Clean Docker system
docker system prune -a
docker volume prune
```

## Local Development (Without Docker)

If you have Anchor 0.28 and Solana 1.16 installed locally:

```bash
make build-local
make deploy-local
```

## CI/CD Integration

For CI/CD pipelines, use:

```yaml
# Example GitHub Actions
- name: Build Anchor Programs
  run: |
    cd programs
    docker compose build anchor-build
    docker compose run --rm anchor-build
```

## File Structure

```
programs/
├── Dockerfile              # Docker build image definition
├── docker-compose.yml      # Docker Compose configuration
├── build.sh                # Build script
├── Makefile                # Make targets
├── .dockerignore           # Files to exclude from Docker context
└── DOCKER_BUILD.md         # This file
```

## Next Steps After Build

1. **Verify Build**: Check `target/deploy/` for compiled `.so` files
2. **Deploy**: `make deploy` to deploy to devnet
3. **Update Config**: Update `src/contracts/config.ts` with deployed program IDs
4. **Test**: Use the frontend hooks to test interactions

## Support

For issues or questions:
- Check `programs/README.md` for program documentation
- Check `DEPLOYMENT.md` in this folder for deployment details
- Review Anchor docs: https://www.anchor-lang.com/docs
