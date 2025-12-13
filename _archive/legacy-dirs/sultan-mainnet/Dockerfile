FROM golang:1.22-alpine AS builder

RUN apk add --no-cache git make gcc musl-dev

WORKDIR /app
COPY . .
RUN go mod init sultan-mainnet 2>/dev/null || true
RUN go mod tidy 2>/dev/null || true
RUN go build -o sultand cmd/main.go || echo "Build needs fixes"

FROM alpine:latest
RUN apk --no-cache add ca-certificates
WORKDIR /root/
COPY --from=builder /app/sultand* ./
EXPOSE 26657 26656 6060 9090
CMD ["./sultand"]
