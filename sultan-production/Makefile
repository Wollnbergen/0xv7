#!/usr/bin/make -f

VERSION := $(shell echo $(shell git describe --tags --always 2>/dev/null || echo "v0.1.0") | sed 's/^v//')
COMMIT := $(shell git log -1 --format='%H' 2>/dev/null || echo "dev")
BUILDDIR ?= $(CURDIR)/build

export GO111MODULE = on

build_tags = netgo
build_tags += $(BUILD_TAGS)

ldflags = -X github.com/cosmos/cosmos-sdk/version.Name=sultan \
    -X github.com/cosmos/cosmos-sdk/version.AppName=sultand \
    -X github.com/cosmos/cosmos-sdk/version.Version=$(VERSION) \
    -X github.com/cosmos/cosmos-sdk/version.Commit=$(COMMIT)

BUILD_FLAGS := -tags "$(build_tags)" -ldflags '$(ldflags)'

all: install

install: go.sum
    @echo "🚀 Installing sultand..."
    @go install -mod=readonly $(BUILD_FLAGS) ./cmd/sultand

build: go.sum
    @echo "🔨 Building sultand..."
    @go build -mod=readonly $(BUILD_FLAGS) -o $(BUILDDIR)/sultand ./cmd/sultand

go.sum: go.mod
    @echo "📦 Ensuring dependencies..."
    @go mod verify || true
    @go mod tidy

clean:
    rm -rf $(BUILDDIR)

run: build
    $(BUILDDIR)/sultand start --minimum-gas-prices=0stake

.PHONY: all build install clean run
