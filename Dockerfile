# ---- Stage 1: Build FAISS and your Rust app ----
FROM debian:bookworm-slim AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    git \
    curl \
    pkg-config \
    libopenblas-dev \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*


# Build FAISS
WORKDIR /opt
RUN git clone https://github.com/facebookresearch/faiss.git
WORKDIR /opt/faiss
RUN cmake -B build \
  -DFAISS_ENABLE_PYTHON=OFF \
  -DFAISS_ENABLE_GPU=OFF \
  -DBUILD_TESTING=OFF \
  -DCMAKE_BUILD_TYPE=Release \
  -DFAISS_ENABLE_C_API=ON \
  -DBUILD_SHARED_LIBS=ON \
  .
RUN cmake --build build -j$(nproc)
RUN cmake --install build

# Install Rust via rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"


# Copy Rust project into container (assumes Dockerfile is in project root)
WORKDIR /app
COPY . .

# Build Rust project
RUN cargo build --release

# ---- Stage 2: Minimal runtime image ----
FROM debian:bookworm-slim AS runtime

# Install only runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libopenblas0 \
    ca-certificates \
    libgomp1 \
    && rm -rf /var/lib/apt/lists/*

# Copy FAISS runtime artifacts and built binary
COPY --from=builder /usr/local /usr/local
COPY --from=builder /app/target/release/semcache-rs /usr/local/bin/semcache-rs

WORKDIR /app
COPY ./assets ./assets

# Point linker to faiss installation
ENV LD_LIBRARY_PATH="/usr/local/lib"

# Start the app
ENTRYPOINT ["/usr/local/bin/semcache-rs"]
