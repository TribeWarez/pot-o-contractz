#!/bin/bash
# Tribewarez DeFi Smart Contracts - Docker Build Script

set -e

echo "🚀 Building Tribewarez DeFi Smart Contracts"
echo "============================================"

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if docker compose is available
if ! docker compose version &> /dev/null; then
    echo "❌ docker compose is not available. Please install Docker Compose (plugin: docker compose)."
    exit 1
fi

# Build the Docker image
echo "📦 Building Docker image..."
docker compose build anchor-build

# Run the build
echo "🔨 Building Anchor programs..."
docker compose run --rm anchor-build

echo ""
echo "✅ Build complete! Programs are in target/deploy/"
echo ""
echo "To deploy:"
echo "  docker compose run --rm anchor-shell anchor deploy"
echo ""
echo "To open an interactive shell:"
echo "  docker compose run --rm anchor-shell"
