#!/bin/bash

echo "🔧 Fixing your existing Cosmos SDK setup..."

cd /workspaces/0xv7/sovereign-chain/sovereign

# Fix the Makefile
cat > Makefile.fixed << 'EOF'
BRANCH := $(shell git rev-parse --abbrev-ref HEAD)
COMMIT := $(shell git log -1 --format='%H')

ldflags = -X github.com/cosmos/cosmos-sdk/version.Name=sovereign \
    -X github.com/cosmos/cosmos-sdk/version.ServerName=sovereignd \
    -X github.com/cosmos/cosmos-sdk/version.Version=$(VERSION) \
    -X github.com/cosmos/cosmos-sdk/version.Commit=$(COMMIT)

BUILD_FLAGS := -ldflags '$(ldflags)'

all: install

install: go.sum
    @echo "Installing sovereignd..."
    @go install -mod=readonly $(BUILD_FLAGS) ./cmd/sovereignd

build:
    @echo "Building sovereignd..."
    @go build -mod=readonly $(BUILD_FLAGS) -o build/sovereignd ./cmd/sovereignd

.PHONY: all install build
EOF

# Create missing cmd files
mkdir -p cmd/sovereignd
cat > cmd/sovereignd/main.go << 'EOF'
package main

import (
    "os"
    "github.com/cosmos/cosmos-sdk/server"
    svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
    "sovereign/app"
    "sovereign/cmd/sovereignd/cmd"
)

func main() {
    rootCmd, _ := cmd.NewRootCmd()
    if err := svrcmd.Execute(rootCmd, "", app.DefaultNodeHome); err != nil {
        os.Exit(1)
    }
}
EOF

# Build it
make build
