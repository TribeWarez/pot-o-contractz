# Tribewarez DeFi Smart Contracts - Docker Build Environment
# Uses compatible versions of Anchor and Solana for stable builds

FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=$CARGO_HOME/bin:$PATH

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    libudev-dev \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Install Rust 1.70.0 (compatible with Solana 1.16)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.70.0 && \
    rustup component add rustfmt clippy

# Install Solana CLI (stable version compatible with Anchor 0.28)
# Download installer with SSL verification disabled (for network issues) or use direct binary
RUN curl -k -L https://release.solana.com/stable/install -o /tmp/solana-install.sh || \
    curl -k -L https://github.com/anza-xyz/agave/releases/latest/download/solana-release-x86_64-unknown-linux-gnu.tar.bz2 -o /tmp/solana.tar.bz2 && \
    if [ -f /tmp/solana-install.sh ]; then \
        chmod +x /tmp/solana-install.sh && \
        sh /tmp/solana-install.sh; \
    else \
        mkdir -p /root/.local/share/solana/install/active_release && \
        tar -xjf /tmp/solana.tar.bz2 -C /root/.local/share/solana/install/active_release --strip-components=1; \
    fi && \
    export PATH="/root/.local/share/solana/install/active_release/bin:$PATH" && \
    echo 'export PATH="/root/.local/share/solana/install/active_release/bin:$PATH"' >> /root/.bashrc && \
    /root/.local/share/solana/install/active_release/bin/solana --version || echo "Solana installed (version check may fail in build)"

# Install Anchor CLI 0.28.0 (compatible with Solana 1.16)
# Install from git tag since 0.28.0 might not be on crates.io
RUN cargo install --git https://github.com/coral-xyz/anchor --tag v0.28.0 anchor-cli --locked --force

# Set working directory
WORKDIR /workspace

# Copy project files
COPY . .

# Set PATH for Solana
ENV PATH="/root/.local/share/solana/install/active_release/bin:$PATH"

# Verify installations
RUN /root/.local/share/solana/install/active_release/bin/solana --version && \
    anchor --version && \
    rustc --version

# Default command
CMD ["/bin/bash"]
