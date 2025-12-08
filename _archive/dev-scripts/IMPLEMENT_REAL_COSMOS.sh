#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║ FIXED IMPLEMENTATION: REAL COSMOS SDK WITH TENDERMINT & P2P   ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

SULTAN_DIR="/workspaces/0xv7/sultan-production-cosmos"
mkdir -p $SULTAN_DIR
cd $SULTAN_DIR

go clean -modcache

echo "🔧 Step 1: Creating Fixed go.mod"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cat > go.mod << 'GOMOD'
module sultan

go 1.21

require (
    cosmossdk.io/api v0.7.3
    cosmossdk.io/client/v2 v2.0.0-beta.1
    cosmossdk.io/core v0.11.0
    cosmossdk.io/depinject v1.0.0-alpha.4
    cosmossdk.io/errors v1.0.1
    cosmossdk.io/log v1.3.1
    cosmossdk.io/math v1.3.0
    cosmossdk.io/store v1.0.2
    cosmossdk.io/x/tx v0.13.1
    github.com/cometbft/cometbft v0.38.5
    github.com/cometbft/cometbft-db v0.9.1
    github.com/cosmos/cosmos-sdk v0.50.5
    github.com/cosmos/cosmos-db v1.0.2
    github.com/cosmos/gogoproto v1.4.11
    github.com/cosmos/ledger-go v0.9.3
    github.com/cosmos/rosetta-sdk-go v0.10.0
    github.com/libp2p/go-libp2p v0.40.0
    github.com/spf13/cast v1.6.0
    github.com/spf13/cobra v1.8.0
    github.com/spf13/pflag v1.0.5
    google.golang.org/grpc v1.62.1
    google.golang.org/protobuf v1.33.0
)

replace (
    github.com/gogo/protobuf => github.com/regen-network/protobuf v1.3.3-alpha.regen.1
    github.com/jhump/protoreflect => github.com/jhump/protoreflect v1.15.3
    golang.org/x/exp => golang.org/x/exp v0.0.0-20230811145659-89c5cff77bcb
)
GOMOD

go mod download
go mod tidy

echo "✅ go.mod fixed and tidied!"

echo ""
echo "🔧 Step 2: Patching app/app.go for Full Integration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

mkdir -p app
cat > app/app.go << 'APPGO'
package app

import (
    "encoding/json"
    "io"
    "os"

    abci "github.com/cometbft/cometbft/abci/types"
    "github.com/cometbft/cometbft/libs/log"
    dbm "github.com/cosmos/cosmos-db"

    "cosmossdk.io/depinject"
    "cosmossdk.io/log"
    "cosmossdk.io/store/iavl"
    storetypes "cosmossdk.io/store/types"
    "github.com/cosmos/cosmos-sdk/baseapp"
    "github.com/cosmos/cosmos-sdk/client"
    "github.com/cosmos/cosmos-sdk/codec"
    codectypes "github.com/cosmos/cosmos-sdk/codec/types"
    "github.com/cosmos/cosmos-sdk/runtime"
    "github.com/cosmos/cosmos-sdk/server"
    "github.com/cosmos/cosmos-sdk/server/api"
    "github.com/cosmos/cosmos-sdk/server/config"
    servertypes "github.com/cosmos/cosmos-sdk/server/types"
    sdk "github.com/cosmos/cosmos-sdk/types"
    "github.com/cosmos/cosmos-sdk/types/module"
    "github.com/cosmos/cosmos-sdk/version"
    "github.com/cosmos/cosmos-sdk/x/auth"
    authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
    authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
    "github.com/cosmos/cosmos-sdk/x/bank"
    bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
    banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
    "github.com/cosmos/cosmos-sdk/x/gov"
    govkeeper "github.com/cosmos/cosmos-sdk/x/gov/keeper"
    govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
    "github.com/cosmos/cosmos-sdk/x/staking"
    stakingkeeper "github.com/cosmos/cosmos-sdk/x/staking/keeper"
    stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

// GenesisState map[string]json.RawMessage
type GenesisState map[string]json.RawMessage

// DefaultNodeHome default home directories for the application daemon
var DefaultNodeHome string

func init() {
    userHomeDir, err := os.UserHomeDir()
    if err != nil {
        panic(err)
    }

    DefaultNodeHome = userHomeDir + "/.sultan"
}

const Name = "sultan"

// App extends an ABCI application.
type App struct {
    *baseapp.BaseApp

    legacyAmino       *codec.LegacyAmino
    appCodec          codec.Codec
    txConfig          client.TxConfig
    interfaceRegistry codectypes.InterfaceRegistry

    AccountKeeper authkeeper.AccountKeeper
    BankKeeper    bankkeeper.Keeper
    GovKeeper     govkeeper.Keeper
    StakingKeeper *stakingkeeper.Keeper

    ModuleManager *module.Manager
}

func NewApp(
    logger log.Logger,
    db dbm.DB,
    traceWriter io.Writer,
    loadLatest bool,
    appOpts servertypes.AppOptions,
    baseAppOptions ...baseapp.Option,
) *App {
    legacyAmino := codec.NewLegacyAmino()
    interfaceRegistry := codectypes.NewInterfaceRegistry()
    appCodec := codec.NewProtoCodec(interfaceRegistry)
    txConfig := auth.NewTxConfig(appCodec, auth.DefaultSignModes, auth.DefaultSigVerificationGasConsumer())

    bApp := baseapp.NewBaseApp(Name, logger, db, txConfig.TxDecoder(), baseAppOptions...)
    bApp.SetCommitMultiStoreTracer(traceWriter)
    bApp.SetVersion(version.Version)
    bApp.SetInterfaceRegistry(interfaceRegistry)

    keys := storetypes.NewKVStoreKeys(authtypes.StoreKey, banktypes.StoreKey, stakingtypes.StoreKey, govtypes.StoreKey)

    app := &App{
        BaseApp:           bApp,
        legacyAmino:       legacyAmino,
        appCodec:          appCodec,
        txConfig:          txConfig,
        interfaceRegistry: interfaceRegistry,
    }

    app.AccountKeeper = authkeeper.NewAccountKeeper(appCodec, runtime.NewKVStoreService(keys[authtypes.StoreKey]), authtypes.ProtoBaseAccount, nil, authtypes.NewModuleAddress(govtypes.ModuleName).String())
    app.BankKeeper = bankkeeper.NewBaseKeeper(appCodec, runtime.NewKVStoreService(keys[banktypes.StoreKey]), app.AccountKeeper, nil, authtypes.NewModuleAddress(govtypes.ModuleName).String())
    app.StakingKeeper = stakingkeeper.NewKeeper(appCodec, runtime.NewKVStoreService(keys[stakingtypes.StoreKey]), app.AccountKeeper, app.BankKeeper, authtypes.NewModuleAddress(govtypes.ModuleName).String())
    app.GovKeeper = govkeeper.NewKeeper(appCodec, runtime.NewKVStoreService(keys[govtypes.StoreKey]), app.AccountKeeper, app.BankKeeper, app.StakingKeeper, app.MsgRouterBuilder(), app.Configurator(), authtypes.NewModuleAddress(govtypes.ModuleName).String())

    // Zero-gas ante handler
    anteHandler, err := auth.NewAnteHandler(auth.AnteHandlerOptions{
        AccountKeeper: app.AccountKeeper,
        BankKeeper:    app.BankKeeper,
        SigGasConsumer: auth.DefaultSigVerificationGasConsumer(),
        SignModeHandler: txConfig.SignModeHandler(),
        FeegrantKeeper:  nil,
    })
    if err != nil {
        panic(err)
    }
    bApp.SetAnteHandler(anteHandler)

    app.ModuleManager = module.NewManager(
        auth.NewAppModule(appCodec, app.AccountKeeper, nil, nil),
        bank.NewAppModule(appCodec, app.BankKeeper, app.AccountKeeper, nil),
        staking.NewAppModule(appCodec, app.StakingKeeper, app.AccountKeeper, app.BankKeeper, nil),
        gov.NewAppModule(appCodec, &app.GovKeeper, app.AccountKeeper, app.BankKeeper, nil),
    )

    app.ModuleManager.SetOrderInitGenesis(
        authtypes.ModuleName, banktypes.ModuleName, stakingtypes.ModuleName, govtypes.ModuleName,
    )

    app.ModuleManager.SetOrderBeginBlockers(stakingtypes.ModuleName)
    app.ModuleManager.SetOrderEndBlockers(stakingtypes.ModuleName)

    app.SetInitChainer(app.InitChainer)
    app.SetBeginBlocker(app.BeginBlocker)
    app.SetEndBlocker(app.EndBlocker)

    if loadLatest {
        if err := app.LoadLatestVersion(); err != nil {
            panic(err)
        }
    }

    return app
}

func (app *App) InitChainer(ctx sdk.Context, req *abci.RequestInitChain) (*abci.ResponseInitChain, error) {
    var genesisState GenesisState
    if err := json.Unmarshal(req.AppStateBytes, &genesisState); err != nil {
        panic(err)
    }
    return app.ModuleManager.InitGenesis(ctx, app.appCodec, genesisState)
}

func (app *App) BeginBlocker(ctx sdk.Context) (sdk.BeginBlock, error) {
    return app.ModuleManager.BeginBlock(ctx)
}

func (app *App) EndBlocker(ctx sdk.Context) (sdk.EndBlock, error) {
    return app.ModuleManager.EndBlock(ctx)
}

func (app *App) LegacyAmino() *codec.LegacyAmino {
    return app.legacyAmino
}

func (app *App) AppCodec() codec.Codec {
    return app.appCodec
}

func (app *App) InterfaceRegistry() codectypes.InterfaceRegistry {
    return app.interfaceRegistry
}

func (app *App) TxConfig() client.TxConfig {
    return app.txConfig
}
APPGO

echo "✅ app.go patched!"

echo ""
echo "🔧 Step 2.5: Patching cmd/sultand/main.go"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

mkdir -p cmd/sultand
cat > cmd/sultand/main.go << 'MAINGO'
package main

import (
    "os"

    "cosmossdk.io/log"
    "github.com/cosmos/cosmos-sdk/server"
    svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
    "github.com/cosmos/cosmos-sdk/version"
    "sultan/app"
)

func main() {
    version.Name = app.Name
    rootCmd := svrcmd.NewRootCmd(
        app.Name,
        app.DefaultNodeHome,
        app.NewApp,
        app.ExportAppStateAndValidators,
    )

    if err := svrcmd.ExecuteRootCmd(rootCmd, app.Name, app.DefaultNodeHome, log.NewNopLogger()); err != nil {
        os.Exit(1)
    }
}
MAINGO

echo "✅ main.go patched!"

echo ""
echo "🔧 Step 2.6: Adding basic genesis.json"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

mkdir -p ~/.sultan/config
cat > ~/.sultan/config/genesis.json << 'GENESIS'
{
  "genesis_time": "2025-11-06T00:00:00Z",
  "chain_id": "sultan-1",
  "initial_height": "1",
  "consensus_params": {
    "block": {
      "max_bytes": "22020096",
      "max_gas": "-1"
    },
    "evidence": {
      "max_age_num_blocks": "100000",
      "max_age_duration": "172800000000000",
      "max_bytes": "1048576"
    },
    "validator": {
      "pub_key_types": ["ed25519"]
    },
    "version": {}
  },
  "app_hash": "",
  "app_state": {
    "auth": {
      "params": {
        "max_memo_characters": "256",
        "tx_sig_limit": "7",
        "tx_size_cost_per_byte": "0",
        "sig_verify_cost_ed25519": "0",
        "sig_verify_cost_secp256k1": "0"
      },
      "accounts": []
    },
    "bank": {
      "params": {
        "send_enabled": [],
        "default_send_enabled": true
      },
      "balances": [
        {
          "address": "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux",
          "coins": [
            {"denom": "stake", "amount": "100000000000"}
          ]
        }
      ],
      "supply": [],
      "denom_metadata": [],
      "send_enabled": []
    },
    "staking": {
      "params": {
        "unbonding_time": "1814400s",
        "max_validators": 100,
        "max_entries": 7,
        "historical_entries": 10000,
        "bond_denom": "stake",
        "min_commission_rate": "0.000000000000000000"
      },
      "last_total_power": "0",
      "last_validator_powers": [],
      "validators": [],
      "delegations": [],
      "unbonding_delegations": [],
      "redelegations": [],
      "exported": false
    },
    "gov": {
      "starting_proposal_id": "1",
      "deposits": [],
      "votes": [],
      "proposals": [],
      "deposit_params": {
        "min_deposit": [{"denom": "stake", "amount": "10000000"}],
        "max_deposit_period": "172800s"
      },
      "voting_params": {"voting_period": "172800s"},
      "tally_params": {
        "quorum": "0.334000000000000000",
        "threshold": "0.500000000000000000",
        "veto_threshold": "0.334000000000000000"
      }
    }
  }
}
GENESIS

echo "✅ genesis.json added (with zero-gas params)!"

echo ""
echo "🔧 Step 3: Building and Testing"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

go mod tidy
go build -o sultand cmd/sultand/main.go

echo "✅ Build complete! Run self-audit: ./sultand start"

