
# Contains prebuilt FAISS c++ and Rust
FROM ghcr.io/sensoris/faiss-base-image:latest AS builder

COPY . /app
WORKDIR /app

# Build release version of project
RUN cargo build --release


# Create runtime image with minimal dependencies
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    libopenblas0 \
    ca-certificates \
    libgomp1 \
    && rm -rf /var/lib/apt/lists/*

# Copy FAISS runtime artifacts, built binary, and project files needed at runtime
COPY --from=builder /usr/local /usr/local
COPY --from=builder /app/target/release/semcache-rs /usr/local/bin/semcache-rs
COPY ./assets ./app/assets
COPY ./config.yaml ./app/config.yaml

# Cached vector embedding models
COPY ./.fastembed_cache ./app/.fastembed_cache

# Point linker to faiss installation
ENV LD_LIBRARY_PATH="/usr/local/lib"

# Change workdir to directory of the binary
WORKDIR /app

# Start the app
ENTRYPOINT ["/usr/local/bin/semcache-rs"]