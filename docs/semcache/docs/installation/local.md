---
sidebar_position: 2
---

# Local Development Setup

Build and run semcache from source for development or custom deployments.

## Prerequisites

### System Requirements
- **Rust**: Latest stable version (1.70+)
- **CMake**: 3.17 or later
- **C++ Compiler**: GCC 7+ or Clang 5+
- **OpenBLAS**: For FAISS performance
- **Git**: For cloning repositories

### Platform Support
- Linux (Ubuntu 20.04+, CentOS 8+)
- macOS (Intel and Apple Silicon)
- Windows (with WSL2 recommended)

## Install Dependencies

### Ubuntu/Debian
```bash
sudo apt update
sudo apt install -y \
  build-essential \
  cmake \
  libopenblas-dev \
  pkg-config \
  curl \
  git
```

### macOS
```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install cmake openblas pkg-config
```

### Windows (WSL2)
```bash
# Inside WSL2 Ubuntu
sudo apt update
sudo apt install -y \
  build-essential \
  cmake \
  libopenblas-dev \
  pkg-config \
  curl \
  git
```

## Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

## Build FAISS

semcache requires FAISS (Facebook AI Similarity Search) to be built and installed.

### Clone FAISS

```bash
git clone https://github.com/facebookresearch/faiss.git
cd faiss
```

### Configure Build

```bash
cmake -B build \
  -DFAISS_ENABLE_PYTHON=OFF \
  -DFAISS_ENABLE_GPU=OFF \
  -DBUILD_TESTING=OFF \
  -DCMAKE_BUILD_TYPE=Release \
  -DFAISS_ENABLE_C_API=ON \
  -DBUILD_SHARED_LIBS=ON \
  .
```

### Build FAISS

```bash
# Build (use appropriate number of cores)
cmake --build build -j$(nproc)
```

### Install FAISS

```bash
# Install system-wide
sudo cmake --install build

# Or install to custom prefix
cmake --install build --prefix /usr/local
```

### Verify FAISS Installation

```bash
# Check if libraries are installed
ls /usr/local/lib/libfaiss*

# Check headers
ls /usr/local/include/faiss/
```

## Build semcache

### Clone Repository

```bash
git clone https://github.com/sensoris/semcache.git
cd semcache
```

### Set Environment Variables

If FAISS is installed in a non-standard location:

```bash
export PKG_CONFIG_PATH="/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH"
export LD_LIBRARY_PATH="/usr/local/lib:$LD_LIBRARY_PATH"
```

### Build semcache

```bash
# Build in release mode
cargo build --release

# The binary will be at: target/release/semcache-rs
```

## Run semcache

```bash
# Run the built binary
./target/release/semcache-rs
```

semcache will start on `http://localhost:8080`.

## Development Workflow

### Development Build

For faster compilation during development:

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Run with cargo
cargo run
```

### Auto-reload on Changes

Install `cargo-watch` for automatic rebuilds:

```bash
cargo install cargo-watch

# Auto-reload on file changes
cargo watch -x run
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_embedding_generation
```

## IDE Setup

### Visual Studio Code

Install recommended extensions:
- **rust-analyzer**: Language server
- **CodeLLDB**: Debugging support
- **Better TOML**: Cargo.toml syntax highlighting

### CLion/IntelliJ

Install:
- **Rust plugin**: Full Rust support
- **TOML plugin**: Configuration file support

## Troubleshooting

### FAISS Build Issues

**CMake not found:**
```bash
# Ubuntu/Debian
sudo apt install cmake

# macOS
brew install cmake
```

**OpenBLAS not found:**
```bash
# Ubuntu/Debian
sudo apt install libopenblas-dev

# macOS
brew install openblas
```

**Permission denied during install:**
```bash
# Use sudo for system-wide install
sudo cmake --install build
```

### Rust Build Issues

**Linker errors:**
```bash
# Make sure development tools are installed
sudo apt install build-essential

# On macOS, install Xcode command line tools
xcode-select --install
```

**FAISS not found:**
```bash
# Make sure FAISS is in library path
export LD_LIBRARY_PATH="/usr/local/lib:$LD_LIBRARY_PATH"

# Or set PKG_CONFIG_PATH
export PKG_CONFIG_PATH="/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH"
```

### Runtime Issues

**Library not found:**
```bash
# Add to shell profile (.bashrc, .zshrc)
export LD_LIBRARY_PATH="/usr/local/lib:$LD_LIBRARY_PATH"
```

**Port already in use:**
```bash
# Check what's using port 8080
lsof -i :8080

# Kill the process or change semcache port (future configuration)
```

## Performance Optimization

### Release Build
Always use release builds for production:

```bash
cargo build --release
```

### CPU Optimization
Enable CPU-specific optimizations:

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### Memory Settings
For large-scale deployments, consider system memory limits and FAISS configuration.

## Next Steps

- [Configuration](../configuration/cache-settings.md) - Configure cache behavior
- [API Reference](../api/chat-completions.md) - Learn the API
- [Monitoring](../monitoring/metrics.md) - Set up monitoring