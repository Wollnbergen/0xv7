package cmd

import (
	"errors"
	"io"
	"os"
	"time"
	
	"cosmossdk.io/log"
	confixcmd "cosmossdk.io/tools/confix/cmd"
	
	cmtcfg "github.com/cometbft/cometbft/config"
	dbm "github.com/cosmos/cosmos-db"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/config"
	"github.com/cosmos/cosmos-sdk/client/debug"
	"github.com/cosmos/cosmos-sdk/client/keys"
	"github.com/cosmos/cosmos-sdk/server"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
	authcodec "github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/x/crisis"
	genutilcli "github.com/cosmos/cosmos-sdk/x/genutil/client/cli"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
	
	"github.com/spf13/cobra"
	
	"github.com/wollnbergen/sultand/app"
)

// NewRootCmd creates a new root command for sultand
func NewRootCmd() *cobra.Command {
	// Set up config
	initClientCtx := client.Context{}.
		WithCodec(app.MakeEncodingConfig().Codec).
		WithInterfaceRegistry(app.MakeEncodingConfig().InterfaceRegistry).
		WithTxConfig(app.MakeEncodingConfig().TxConfig).
		WithLegacyAmino(app.MakeEncodingConfig().Amino).
		WithInput(os.Stdin).
		WithAccountRetriever(types.AccountRetriever{}).
		WithHomeDir(app.DefaultNodeHome).
		WithViper("")
	
	rootCmd := &cobra.Command{
		Use:   "sultand",
		Short: "Sultan Blockchain Daemon",
		Long: `Sultan is a high-performance blockchain with zero gas fees.
Built with Rust core and integrated with Cosmos SDK via FFI.

Features:
- Zero gas fees on all transactions
- 100k+ transactions per second
- Instant finality
- Cosmos SDK compatibility
- IBC support (planned)`,
		PersistentPreRunE: func(cmd *cobra.Command, _ []string) error {
			// Set output writer
			cmd.SetOut(cmd.OutOrStdout())
			cmd.SetErr(cmd.ErrOrStderr())
			
			initClientCtx = initClientCtx.WithCmdContext(cmd.Context())
			initClientCtx, err := client.ReadPersistentCommandFlags(initClientCtx, cmd.Flags())
			if err != nil {
				return err
			}
			
			initClientCtx, err = config.ReadFromClientConfig(initClientCtx)
			if err != nil {
				return err
			}
			
			if err := client.SetCmdClientContextHandler(initClientCtx, cmd); err != nil {
				return err
			}
			
			customAppTemplate, customAppConfig := initAppConfig()
			customCMTConfig := initCometBFTConfig()
			
			return server.InterceptConfigsPreRunHandler(cmd, customAppTemplate, customAppConfig, customCMTConfig)
		},
	}
	
	initRootCmd(rootCmd, initClientCtx)
	
	return rootCmd
}

func initRootCmd(rootCmd *cobra.Command, clientCtx client.Context) {
	// Use registered module basics for proper genesis initialization
	sdkConfig := sdk.GetConfig()
	addressCodec := authcodec.NewBech32Codec(sdkConfig.GetBech32AccountAddrPrefix())
	validatorAddressCodec := authcodec.NewBech32Codec(sdkConfig.GetBech32ValidatorAddrPrefix())
	
	// Add genesis commands
	rootCmd.AddCommand(
		confixcmd.ConfigCommand(),
		genutilcli.InitCmd(app.ModuleBasics, app.DefaultNodeHome),
		genutilcli.AddGenesisAccountCmd(app.DefaultNodeHome, addressCodec),
		genutilcli.GenTxCmd(app.ModuleBasics, clientCtx.TxConfig, app.GenesisBalancesIterator(), app.DefaultNodeHome, validatorAddressCodec),
		genutilcli.CollectGenTxsCmd(app.GenesisBalancesIterator(), app.DefaultNodeHome, genutiltypes.DefaultMessageValidator, validatorAddressCodec),
		genutilcli.ValidateGenesisCmd(app.ModuleBasics),
		debug.Cmd(),
	)
	
	// Server commands
	server.AddCommands(rootCmd, app.DefaultNodeHome, newApp, appExport, addModuleInitFlags)
	
	// Transaction commands
	rootCmd.AddCommand(
		TxCmd(),
		QueryCmd(),
		keys.Commands(),
	)
}

func addModuleInitFlags(startCmd *cobra.Command) {
	crisis.AddModuleInitFlags(startCmd)
}

// newApp creates a new Sultan application
func newApp(
	logger log.Logger,
	db dbm.DB,
	traceStore io.Writer,
	appOpts servertypes.AppOptions,
) servertypes.Application {
	baseappOptions := server.DefaultBaseappOptions(appOpts)
	
	return app.NewSultanApp(
		logger,
		db,
		traceStore,
		true,
		appOpts,
		baseappOptions...,
	)
}

// appExport creates a new Sultan app exporter
func appExport(
	logger log.Logger,
	db dbm.DB,
	traceStore io.Writer,
	height int64,
	forZeroHeight bool,
	jailAllowedAddrs []string,
	appOpts servertypes.AppOptions,
	modulesToExport []string,
) (servertypes.ExportedApp, error) {
	// For now, return error - implement export logic in production
	return servertypes.ExportedApp{}, errors.New("export not implemented yet")
}

func initCometBFTConfig() *cmtcfg.Config {
	// Create default CometBFT config
	cfg := cmtcfg.DefaultConfig()
	
	// Customize for Sultan - fast finality
	cfg.Consensus.TimeoutPropose = time.Second
	cfg.Consensus.TimeoutPrevote = time.Second
	cfg.Consensus.TimeoutPrecommit = time.Second
	cfg.Consensus.TimeoutCommit = time.Second
	
	return cfg
}

func initAppConfig() (string, interface{}) {
	// Return basic app config template
	customTemplate := `
# Sultan minimum gas prices (zero fees!)
minimum-gas-prices = "0sultan"
`
	
	type CustomAppConfig struct {
		MinGasPrices string
	}
	
	srvCfg := CustomAppConfig{
		MinGasPrices: "0sultan",
	}
	
	return customTemplate, srvCfg
}
