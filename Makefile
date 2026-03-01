.PHONY: build deploy test clean shell help

# Default target
help:
	@echo "Tribewarez DeFi Smart Contracts - Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  make build    - Build all Anchor programs using Docker"
	@echo "  make deploy    - Deploy programs to devnet"
	@echo "  make test      - Run tests"
	@echo "  make clean     - Clean build artifacts"
	@echo "  make shell     - Open interactive Docker shell"
	@echo "  make help      - Show this help message"

# Build programs
build:
	@echo "🔨 Building Anchor programs..."
	@if docker compose build anchor-build 2>/dev/null; then \
		docker compose run --rm anchor-build; \
	else \
		echo "⚠️  Docker build failed, trying alternative method..."; \
		docker build -f Dockerfile.alternative -t tribewarez-anchor-build . && \
		docker run --rm -v $$(pwd):/workspace tribewarez-anchor-build \
			sh -c "cd /workspace/programs && anchor build"; \
	fi

# Deploy to devnet
deploy:
	@echo "🚀 Deploying to devnet..."
	@docker compose run --rm anchor-shell anchor deploy --provider.cluster devnet

# Run tests
test:
	@echo "🧪 Running tests..."
	@docker compose run --rm anchor-shell anchor test

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@docker compose run --rm anchor-shell sh -c "cd programs && rm -rf target Cargo.lock"

# Open interactive shell
shell:
	@echo "🐚 Opening interactive shell..."
	@docker compose run --rm anchor-shell

# Build without Docker (requires local Anchor/Solana)
build-local:
	@echo "🔨 Building locally (requires Anchor 0.28 and Solana 1.16)..."
	@cd programs && anchor build

# Deploy locally
deploy-local:
	@echo "🚀 Deploying locally..."
	@cd programs && anchor deploy --provider.cluster devnet
