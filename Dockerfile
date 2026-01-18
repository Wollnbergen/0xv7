# Build stage
FROM rust:1.85 AS builder

# Install build dependencies (libclang for bindgen, cmake for native libs)
RUN apt-get update && apt-get install -y \
    libclang-dev \
    clang \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY sultan-core ./sultan-core

# Build the release binary
RUN cargo build --release --package sultan-core --bin sultan-node

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sultan-node /usr/local/bin/

EXPOSE 26656 26657

CMD ["sultan-node"]
