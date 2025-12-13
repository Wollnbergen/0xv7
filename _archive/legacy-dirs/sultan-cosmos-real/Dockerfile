# Multi-stage build for Sultan Cosmos Chain
FROM golang:1.21-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    git \
    build-base \
    linux-headers

WORKDIR /app

# Copy go mod files
COPY go.mod go.sum ./
RUN go mod download

# Copy source code
COPY . .

# Build binary
RUN CGO_ENABLED=0 GOOS=linux go build \
    -ldflags '-w -s -X github.com/cosmos/cosmos-sdk/version.Name=sultan \
              -X github.com/cosmos/cosmos-sdk/version.AppName=sultand \
              -X github.com/cosmos/cosmos-sdk/version.Version=v1.0.0 \
              -X github.com/cosmos/cosmos-sdk/version.Commit=$(git rev-parse HEAD)' \
    -o /app/build/sultand ./cmd/sultand

# Runtime stage
FROM alpine:latest

RUN apk add --no-cache \
    ca-certificates \
    bash \
    jq \
    curl

# Create sultan user
RUN addgroup -g 1000 sultan && \
    adduser -D -u 1000 -G sultan sultan

WORKDIR /home/sultan

# Copy binary from builder
COPY --from=builder /app/build/sultand /usr/local/bin/
RUN chmod +x /usr/local/bin/sultand

# Switch to sultan user
USER sultan

# Expose ports
# P2P
EXPOSE 26656
# RPC
EXPOSE 26657
# gRPC
EXPOSE 9090
# REST API
EXPOSE 1317
# Prometheus metrics
EXPOSE 26660

# Set default home directory
ENV SULTAN_HOME=/home/sultan/.sultan

# Volume for data persistence
VOLUME ["/home/sultan/.sultan"]

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:26657/health || exit 1

# Default command
ENTRYPOINT ["sultand"]
CMD ["start"]
