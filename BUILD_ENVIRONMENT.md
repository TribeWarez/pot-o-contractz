# Build Environment Setup - Complete Guide

## ✅ Docker-Based Build Environment Created

A complete Docker-based build environment has been set up to ensure consistent, reproducible builds across all platforms (Ubuntu, macOS, Windows, CI/CD).

## 📦 What's Included

### Docker Configuration Files

1. **`Dockerfile`** - Main Docker image definition
   - Ubuntu 22.04 base
   - Rust 1.70.0 (compatible version)
   - Solana CLI (stable release)
   - Anchor CLI 0.28.0

2. **`docker-compose.yml`** - Docker Compose configuration
   - Build service for compiling programs
   - Shell service for interactive development
   - Volume mounts for caching

3. **`build.sh`** - Convenience build script
   - One-command build execution

4. **`Makefile`** - Make targets for common tasks
   - `make build` - Build programs
   - `make deploy` - Deploy to devnet
   - `make test` - Run tests
   - `make shell` - Interactive shell

5. **`.dockerignore`** - Excludes unnecessary files from Docker context

## 🚀 Quick Start

### First Time Setup

```bash
cd programs

# Build the Docker image (takes 10-15 minutes first time)
docker compose build anchor-build

# Build the programs
make build
```

### Subsequent Builds

```bash
# Much faster - uses Docker cache
make build
```

## 📋 Build Process

The Docker build process:

1. **Base Image**: Ubuntu 22.04
2. **Install Rust**: 1.70.0 (compatible with Solana 1.16)
3. **Install Solana**: Latest stable release
4. **Install Anchor**: 0.28.0 from GitHub (compatible version)
5. **Build Programs**: Compile all three Anchor programs
6. **Output**: `.so` files in `target/deploy/`

## 🔧 Environment Details

### Versions Used

- **OS**: Ubuntu 22.04
- **Rust**: 1.70.0
- **Solana**: Latest stable (compatible with Anchor 0.28)
- **Anchor**: 0.28.0

### Why These Versions?

- **Anchor 0.28** is stable and well-tested
- **Solana 1.16** is compatible with Anchor 0.28
- **Rust 1.70** is what Solana BPF toolchain uses internally
- These versions avoid the dependency conflicts we encountered

## 📁 File Structure

```
programs/
├── Dockerfile                 # Docker image definition
├── Dockerfile.optimized      # Optimized version (optional)
├── docker-compose.yml        # Docker Compose config
├── build.sh                  # Build script
├── Makefile                  # Make targets
├── .dockerignore            # Docker ignore patterns
├── DOCKER_BUILD.md          # Docker documentation
├── QUICKSTART.md            # Quick start guide
└── BUILD_ENVIRONMENT.md     # This file
```

## 🎯 Usage Examples

### Build Programs

```bash
# Using Make (recommended)
make build

# Using script
./build.sh

# Using docker compose directly
docker compose run --rm anchor-build
```

### Deploy to Devnet

```bash
make deploy
```

### Interactive Development

```bash
# Open shell in build environment
make shell

# Then run commands
anchor build
anchor test
anchor deploy
```

### Check Versions

```bash
docker compose run --rm anchor-shell anchor --version
docker compose run --rm anchor-shell solana --version
docker compose run --rm anchor-shell rustc --version
```

## 🔍 Troubleshooting

### Build Takes Too Long

**First build**: 10-15 minutes (normal - installing all dependencies)
**Subsequent builds**: 1-2 minutes (uses Docker cache)

### Docker Build Fails

1. Check Docker is running: `docker ps`
2. Check disk space: `df -h`
3. Clean Docker: `docker system prune -a`
4. Rebuild: `docker compose build --no-cache anchor-build`

### Permission Issues

```bash
# Add user to docker group (Linux)
sudo usermod -aG docker $USER
# Log out and back in
```

### Out of Memory

Increase Docker memory limit:
- Docker Desktop: Settings → Resources → Memory
- Docker daemon: Edit `/etc/docker/daemon.json`

## 🚢 CI/CD Integration

### GitHub Actions Example

```yaml
name: Build Anchor Programs

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build programs
        run: |
          cd programs
          docker compose build anchor-build
          docker compose run --rm anchor-build
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: programs
          path: programs/target/deploy/*.so
```

### GitLab CI Example

```yaml
build:
  image: docker:latest
  services:
    - docker:dind
  script:
    - cd programs
    - docker compose build anchor-build
    - docker compose run --rm anchor-build
  artifacts:
    paths:
      - programs/target/deploy/*.so
```

## 📊 Build Output

After successful build:

```
target/
├── deploy/
│   ├── tribewarez_staking.so
│   ├── tribewarez_vault.so
│   ├── tribewarez_swap.so
│   └── *-keypair.json files
└── idl/
    ├── tribewarez_staking.json
    ├── tribewarez_vault.json
    └── tribewarez_swap.json
```

## ✅ Verification

After build, verify:

```bash
# Check .so files exist
ls -lh target/deploy/*.so

# Check file sizes (should be > 100KB each)
du -h target/deploy/*.so

# Verify with Solana CLI
solana program dump <program-id> program.so
```

## 🎓 Next Steps

1. ✅ Build environment is ready
2. 🔨 Build programs: `make build`
3. 🚀 Deploy to devnet: `make deploy`
4. 🔧 Update frontend config with deployed program IDs
5. 🧪 Test with frontend hooks
6. 🌐 Deploy to mainnet after testing

## 📚 Additional Resources

- **Docker Build**: `DOCKER_BUILD.md`
- **Quick Start**: `QUICKSTART.md`
- **Deployment**: `DEPLOYMENT.md`
- **Program Docs**: `README.md`

## 💡 Tips

- **Use Make**: `make build` is easier than typing docker compose commands
- **Cache Volumes**: Docker volumes cache cargo registry and Solana installation
- **Parallel Builds**: Docker can build multiple programs in parallel
- **CI/CD**: Use the same Docker setup in CI/CD for consistency

---

**Status**: ✅ Docker build environment is ready to use!
