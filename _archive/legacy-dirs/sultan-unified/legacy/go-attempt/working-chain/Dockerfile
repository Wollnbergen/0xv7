FROM golang:1.21-alpine AS builder

RUN apk add --no-cache git make gcc musl-dev

WORKDIR /app
COPY main.go .
COPY go.mod go.sum* ./
RUN go mod init sultan-chain 2>/dev/null || true
RUN go mod tidy
RUN go build -o sultan-chain main.go

FROM alpine:latest
RUN apk --no-cache add ca-certificates
WORKDIR /root/
COPY --from=builder /app/sultan-chain .
EXPOSE 8080
CMD ["./sultan-chain"]
