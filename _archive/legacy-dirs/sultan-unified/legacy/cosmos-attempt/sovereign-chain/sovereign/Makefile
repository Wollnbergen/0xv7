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
	@echo "Building sovereign chain..."
	go build -o build/sovereign ./cmd/sovereign

go.sum: go.mod
	@echo "--> Ensure dependencies have not been modified"
	@go mod verify

test:
	@go test -mod=readonly $(PACKAGES_SIMTEST)

clean:
	rm -rf $(BUILDDIR)/

.PHONY: all install build go.sum test clean
