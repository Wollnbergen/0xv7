package cmd

import (
    "errors"
    "io"
    "os"

    dbm "github.com/cosmos/cosmos-db"

    "cosmossdk.io/log"
    confixcmd "cosmossdk.io/tools/confix/cmd"
    tmcfg "github.com/cometbft/cometbft/config"
    tmcli "github.com/cometbft/cometbft/libs/cli"
    "github.com/cosmos/cosmos-sdk/client"
    "github.com/cosmos/cosmos-sdk/client/config"
    "github.com/cosmos/cosmos-sdk/client/debug"
    "github.com/cosmos/cosmos-sdk/client/flags"
    "github.com/cosmos/cosmos-sdk/client/keys"
    "github.com/cosmos/cosmos-sdk/client/pruning"
    "github.com/cosmos/cosmos-sdk/server"
    serverconfig "github.com/cosmos/cosmos-sdk/server/config"
    servertypes "github.com/cosmos/cosmos-sdk/server/types"
    authcodec "github.com/cosmos/cosmos-sdk/x/auth/codec"
    banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
    "github.com/cosmos/cosmos-sdk/x/crisis"
    genutilcli "github.com/cosmos/cosmos-sdk/x/genutil/client/cli"
    genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
    "github.com/spf13/cobra"

    "github.com/sultan-chain/sultan/app"
)

// NewRootCmd creates a new root command for sultand
func NewRootCmd() *cobra.Command {
    encodingConfig := app.MakeEncodingConfig()
    initClientCtx := client.Context{}.
        WithCodec(encodingConfig.Codec).
        WithInterfaceRegistry(encodingConfig.InterfaceRegistry).
        WithTxConfig(encodingConfig.TxConfig).
        WithLegacyAmino(encodingConfig.Amino).
        WithInput(os.Stdin).
        WithBroadcastMode(flags.FlagBroadcastMode).
        WithHomeDir(app.DefaultNodeHome).
        WithViper("SULTAN")

    rootCmd := &cobra.Command{
        Use:   "sultand",
        Short: "Sultan Chain - Cosmos SDK Application",
        PersistentPreRunE: func(cmd *cobra.Command, _ []string) error {
            // set the default command outputs
            cmd.SetOut(cmd.OutOrStdout())
            cmd.SetErr(cmd.ErrOrStderr())

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

            customCMTConfig := initCometBFTConfig()
            customAppConfig := initAppConfig()
            customAppTemplate := initAppTemplate()

            return server.InterceptConfigsPreRunHandler(cmd, customAppTemplate, customAppConfig, customCMTConfig)
        },
    }

    initRootCmd(rootCmd, encodingConfig)
    return rootCmd
}

func initRootCmd(rootCmd *cobra.Command, encodingConfig app.EncodingConfig) {
    // SDK config already set and sealed in app package init()
    // Do not seal again or configure here
    
    rootCmd.AddCommand(
        genutilcli.InitCmd(app.ModuleBasics, app.DefaultNodeHome),
        genutilcli.CollectGenTxsCmd(banktypes.GenesisBalancesIterator{}, app.DefaultNodeHome, genutiltypes.DefaultMessageValidator, authcodec.NewBech32Codec(app.AccountAddressPrefix)),
        genutilcli.GenTxCmd(app.ModuleBasics, encodingConfig.TxConfig, banktypes.GenesisBalancesIterator{}, app.DefaultNodeHome, authcodec.NewBech32Codec(app.AccountAddressPrefix)),
        genutilcli.ValidateGenesisCmd(app.ModuleBasics),
        AddGenesisAccountCmd(app.DefaultNodeHome),
        tmcli.NewCompletionCmd(rootCmd, true),
        debug.Cmd(),
        confixcmd.ConfigCommand(),
        pruning.Cmd(newApp, app.DefaultNodeHome),
        keys.Commands(),
    )

    server.AddCommands(rootCmd, app.DefaultNodeHome, newApp, appExport, addModuleInitFlags)
}

func newApp(logger log.Logger, db dbm.DB, traceStore io.Writer, appOpts servertypes.AppOptions) servertypes.Application {
    baseappOptions := server.DefaultBaseappOptions(appOpts)
    return app.NewSultanApp(logger, db, traceStore, true, appOpts, baseappOptions...)
}

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
    // TODO: Implement export functionality
    return servertypes.ExportedApp{}, errors.New("not implemented")
}

func addModuleInitFlags(cmd *cobra.Command) {
    crisis.AddModuleInitFlags(cmd)
}

func initCometBFTConfig() *tmcfg.Config {
    cfg := tmcfg.DefaultConfig()
    // Customize CometBFT config here
    return cfg
}

func initAppConfig() *serverconfig.Config {
    cfg := serverconfig.DefaultConfig()
    cfg.MinGasPrices = "0stake" // Zero gas fees
    return cfg
}

func initAppTemplate() string {
    return serverconfig.DefaultConfigTemplate
}

// AddGenesisAccountCmd returns add-genesis-account cobra Command.
func AddGenesisAccountCmd(defaultNodeHome string) *cobra.Command {
    return genutilcli.AddGenesisAccountCmd(defaultNodeHome, authcodec.NewBech32Codec(app.AccountAddressPrefix))
}
