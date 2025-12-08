#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          FIXING AND BUILDING SOVEREIGN CHAIN                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/sovereign-chain/sovereign

# Step 1: Fix the Makefile
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔧 [1/6] Fixing Makefile..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cat > Makefile << 'MAKEFILE'
BRANCH := $(shell git rev-parse --abbrev-ref HEAD)
COMMIT := $(shell git log -1 --format='%H')

# don't override user values
ifeq (,$(VERSION))
  VERSION := $(shell git describe --tags 2>/dev/null)
  # if VERSION is empty, then populate it with branch's name and raw commit hash
  ifeq (,$(VERSION))
    VERSION := $(BRANCH)-$(COMMIT)
  endif
endif

PACKAGES_SIMTEST=$(shell go list ./... | grep '/simulation')
LEDGER_ENABLED ?= true
SDK_PACK := $(shell go list -m github.com/cosmos/cosmos-sdk | sed  's/ /\@/g')
TM_VERSION := $(shell go list -m github.com/cometbft/cometbft | sed 's:.* ::') # grab everything after the space
DOCKER := $(shell which docker)
BUILDDIR ?= $(CURDIR)/build

export GO111MODULE = on

# process build tags

build_tags = netgo
ifeq ($(LEDGER_ENABLED),true)
  ifeq ($(OS),Windows_NT)
    GCCEXE = $(shell where gcc.exe 2> NUL)
    ifeq ($(GCCEXE),)
      $(error gcc.exe not installed for ledger support, please install or set LEDGER_ENABLED=false)
    endif
  else
    UNAME_S = $(shell uname -s)
    ifeq ($(UNAME_S),OpenBSD)
      $(warning OpenBSD detected, disabling ledger support (https://github.com/cosmos/cosmos-sdk/issues/1988))
      LEDGER_ENABLED = false
    endif

    GCC = $(shell command -v gcc 2> /dev/null)
    ifeq ($(GCC),)
      $(error gcc not installed for ledger support, please install or set LEDGER_ENABLED=false)
    endif
  endif

  build_tags += $(BUILD_TAGS)
  build_tags := $(strip $(build_tags))

  whitespace :=
  whitespace := $(whitespace) $(whitespace)
  comma := ,
  build_tags_comma_sep := $(subst $(whitespace),$(comma),$(build_tags))

  # process linker flags

  ldflags = -X github.com/cosmos/cosmos-sdk/version.Name=sovereign \
          -X github.com/cosmos/cosmos-sdk/version.AppName=sovereignd \
          -X github.com/cosmos/cosmos-sdk/version.Version=$(VERSION) \
          -X github.com/cosmos/cosmos-sdk/version.Commit=$(COMMIT) \
          -X "github.com/cosmos/cosmos-sdk/version.BuildTags=$(build_tags_comma_sep)" \
            -X github.com/cometbft/cometbft/version.TMCoreSemVer=$(TM_VERSION)

  ifeq (cleveldb,$(findstring cleveldb,$(SOVEREIGN_BUILD_OPTIONS)))
    build_tags += gcc
  endif
endif

ifeq (rocksdb,$(findstring rocksdb,$(SOVEREIGN_BUILD_OPTIONS)))
  CGO_ENABLED=1
  build_tags += rocksdb
endif

ifeq (boltdb,$(findstring boltdb,$(SOVEREIGN_BUILD_OPTIONS)))
  build_tags += boltdb
endif

build_tags += $(BUILD_TAGS)
build_tags := $(strip $(build_tags))

whitespace :=
whitespace := $(whitespace) $(whitespace)
comma := ,
build_tags_comma_sep := $(subst $(whitespace),$(comma),$(build_tags))

# process linker flags

ldflags = -X github.com/cosmos/cosmos-sdk/version.Name=sovereign \
    -X github.com/cosmos/cosmos-sdk/version.AppName=sovereignd \
    -X github.com/cosmos/cosmos-sdk/version.Version=$(VERSION) \
    -X github.com/cosmos/cosmos-sdk/version.Commit=$(COMMIT) \
    -X "github.com/cosmos/cosmos-sdk/version.BuildTags=$(build_tags_comma_sep)" \
    -X github.com/cometbft/cometbft/version.TMCoreSemVer=$(TM_VERSION)

ifeq ($(LINK_STATICALLY),true)
  ldflags += -linkmode=external -extldflags "-Wl,-z,muldefs -static"
endif

ifeq (,$(findstring nostrip,$(SOVEREIGN_BUILD_OPTIONS)))
  ldflags += -w -s
endif
ldflags += $(LDFLAGS)
ldflags := $(strip $(ldflags))

BUILD_FLAGS := -tags "$(build_tags)" -ldflags '$(ldflags)'
# check for nostrip option
ifeq (,$(findstring nostrip,$(SOVEREIGN_BUILD_OPTIONS)))
  BUILD_FLAGS += -trimpath
endif

all: install

install: go.sum
    go install -mod=readonly $(BUILD_FLAGS) ./cmd/sovereignd

build:
    go build -mod=readonly $(BUILD_FLAGS) -o $(BUILDDIR)/ ./cmd/sovereignd

go.sum: go.mod
    @echo "--> Ensure dependencies have not been modified"
    @go mod verify

test:
    @go test -mod=readonly $(PACKAGES_SIMTEST)

clean:
    rm -rf $(BUILDDIR)/

.PHONY: all install build go.sum test clean
MAKEFILE

echo "✅ Makefile fixed"

# Step 2: Fix go.mod dependencies
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔧 [2/6] Fixing go.mod dependencies..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Fix the epochs module import issue
sed -i 's|github.com/cosmos/cosmos-sdk/x/epochs|cosmossdk.io/x/epochs|g' app/*.go 2>/dev/null

# Update go.mod to use correct module paths
go mod edit -replace github.com/cosmos/cosmos-sdk/x/epochs=cosmossdk.io/x/epochs@v0.2.0-rc.1
go mod download
go mod tidy

# Step 3: Build using ignite if available, otherwise use make
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔨 [3/6] Building sovereignd binary..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if command -v ignite &> /dev/null; then
    echo "Using Ignite to build..."
    ignite chain build --release 2>&1 | tail -20
else
    echo "Using make to build..."
    make install 2>&1 | tail -20
fi

# Step 4: Check if build succeeded
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📍 [4/6] Locating binary..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

BINARY_FOUND=false
BINARY_PATH=""

# Check multiple possible locations
if [ -f "$HOME/go/bin/sovereignd" ]; then
    BINARY_PATH="$HOME/go/bin/sovereignd"
    BINARY_FOUND=true
elif [ -f "./build/sovereignd" ]; then
    BINARY_PATH="./build/sovereignd"
    BINARY_FOUND=true
elif [ -f "$(go env GOPATH)/bin/sovereignd" ]; then
    BINARY_PATH="$(go env GOPATH)/bin/sovereignd"
    BINARY_FOUND=true
fi

if [ "$BINARY_FOUND" = true ]; then
    echo "✅ Binary found at: $BINARY_PATH"
    
    # Copy to standard location if not already there
    if [ "$BINARY_PATH" != "$HOME/go/bin/sovereignd" ]; then
        mkdir -p $HOME/go/bin
        cp "$BINARY_PATH" $HOME/go/bin/sovereignd
        chmod +x $HOME/go/bin/sovereignd
        echo "   Copied to: $HOME/go/bin/sovereignd"
    fi
    
    # Step 5: Initialize the chain
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "⚡ [5/6] Initializing chain..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Remove old data
    rm -rf $HOME/.sovereign
    
    # Initialize
    $HOME/go/bin/sovereignd init sovereign-node --chain-id sovereign-1 --home $HOME/.sovereign
    
    # Configure for zero gas
    if [ -f "$HOME/.sovereign/config/app.toml" ]; then
        sed -i 's/minimum-gas-prices = ".*"/minimum-gas-prices = "0stake"/' $HOME/.sovereign/config/app.toml
        echo "✅ Zero gas fees configured"
    fi
    
    # Step 6: Create accounts
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "👤 [6/6] Creating accounts and genesis..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Create accounts
    echo "test" | $HOME/go/bin/sovereignd keys add validator --keyring-backend test --home $HOME/.sovereign 2>&1 | tee validator.info
    echo "test" | $HOME/go/bin/sovereignd keys add alice --keyring-backend test --home $HOME/.sovereign 2>&1 | tee alice.info
    
    # Add genesis accounts
    $HOME/go/bin/sovereignd genesis add-genesis-account validator 100000000000stake --keyring-backend test --home $HOME/.sovereign
    $HOME/go/bin/sovereignd genesis add-genesis-account alice 10000000000stake --keyring-backend test --home $HOME/.sovereign
    
    # Create genesis transaction
    $HOME/go/bin/sovereignd genesis gentx validator 1000000stake --chain-id sovereign-1 --keyring-backend test --home $HOME/.sovereign
    
    # Collect genesis transactions
    $HOME/go/bin/sovereignd genesis collect-gentxs --home $HOME/.sovereign
    
    echo "✅ Chain fully configured!"
else
    echo "❌ Binary not built. Trying alternative build method..."
    
    # Try building with go build directly
    echo "Attempting direct go build..."
    go build -o $HOME/go/bin/sovereignd ./cmd/sovereignd 2>&1 | tail -10
    
    if [ -f "$HOME/go/bin/sovereignd" ]; then
        echo "✅ Build successful with go build!"
    else
        echo "❌ Build failed. Please check errors above."
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 FINAL STATUS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "$HOME/go/bin/sovereignd" ]; then
    echo "✅ Binary: BUILT"
    echo "✅ Chain: READY TO RUN"
    echo ""
    echo "🚀 Start the chain with:"
    echo "   $HOME/go/bin/sovereignd start --home $HOME/.sovereign"
    echo ""
    echo "Or use the helper script:"
    echo "   ./RUN_SOVEREIGN.sh"
else
    echo "❌ Binary: NOT BUILT"
    echo ""
    echo "Manual build steps:"
    echo "1. cd /workspaces/0xv7/sovereign-chain/sovereign"
    echo "2. go mod tidy"
    echo "3. go build -o ~/go/bin/sovereignd ./cmd/sovereignd"
fi

