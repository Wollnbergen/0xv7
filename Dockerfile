# Build stage
FROM rust:1.75 as builder

WORKDIR /app
COPY node/Cargo.toml node/Cargo.lock ./
COPY node/src ./src

RUN cargo build --release --bin sultan_node

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sultan_node /usr/local/bin/

EXPOSE 26656 26657

CMD ["sultan_node"]
