package cmd

import (
    "errors"
    "io"
    "os"

    "cosmossdk.io/log"
    confixcmd "cosmossdk.io/tools/confix/cmd"
    dbm "github.com/cometbft/cometbft-db"
    tmcli "github.com/cometbft/cometbft/libs/cli"
    "github.com/cosmos/cosmos-sdk/client"
    "github.com/cosmos/cosmos-sdk/client/config"
    "github.com/cosmos/cosmos-sdk/client/debug"
    "github.com/cosmos/cosmos-sdk/client/flags"
    "github.com/cosmos/cosmos-sdk/client/keys"
    "github.com/cosmos/cosmos-sdk/client/pruning"
    "github.com/cosmos/cosmos-sdk/client/rpc"
    "github.com/cosmos/cosmos-sdk/server"
    servertypes "github.com/cosmos/cosmos-sdk/server/types"
    sdk "github.com/cosmos/cosmos-sdk/types"
    authcmd "github.com/cosmos/cosmos-sdk/x/auth/client/cli"
    "github.com/cosmos/cosmos-sdk/x/auth/types"
    banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
    "github.com/cosmos/cosmos-sdk/x/crisis"
    genutilcli "github.com/cosmos/cosmos-sdk/x/genutil/client/cli"
    "github.com/spf13/cobra"
    
    "sultan/app"
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
        WithAccountRetriever(types.AccountRetriever{}).
        WithBroadcastMode(flags.FlagBroadcastMode).
        WithHomeDir(app.DefaultNodeHome).
        WithViper("")

    rootCmd := &cobra.Command{
        Use:   "sultand",
        Short: "Sultan Chain Daemon",
        PersistentPreRunE: func(cmd *cobra.Command, _ []string) error {
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
            customTMConfig := initConfigAndDefaultGenesis()
            return server.InterceptConfigsPreRunHandler(cmd, "", nil, customTMConfig)
        },
    }

    initRootCmd(rootCmd, encodingConfig)

    return rootCmd
}

func initRootCmd(rootCmd *cobra.Command, encodingConfig app.EncodingConfig) {
    rootCmd.AddCommand(
        genutilcli.InitCmd(app.ModuleManager, app.DefaultNodeHome),
        debug.Cmd(),
        confixcmd.ConfigCommand(),
        pruning.Cmd(newApp, app.DefaultNodeHome),
    )

    server.AddCommands(rootCmd, app.DefaultNodeHome, newApp, appExport, addModuleInitFlags)

    rootCmd.AddCommand(
        rpc.StatusCommand(),
        queryCommand(),
        txCommand(),
        keys.Commands(),
    )
}

func queryCommand() *cobra.Command {
    cmd := &cobra.Command{
        Use:     "query",
        Aliases: []string{"q"},
        Short:   "Querying subcommands",
    }

    cmd.AddCommand(
        rpc.ValidatorCommand(),
        rpc.BlockCommand(),
        authcmd.QueryTxsByEventsCmd(),
        authcmd.QueryTxCmd(),
    )

    return cmd
}

func txCommand() *cobra.Command {
    cmd := &cobra.Command{
        Use:   "tx",
        Short: "Transactions subcommands",
    }

    cmd.AddCommand(
        authcmd.GetSignCommand(),
        authcmd.GetSignBatchCommand(),
        authcmd.GetMultiSignCommand(),
        authcmd.GetValidateSignaturesCommand(),
        authcmd.GetBroadcastCommand(),
        authcmd.GetEncodeCommand(),
        authcmd.GetDecodeCommand(),
    )

    return cmd
}

func newApp(logger log.Logger, db dbm.DB, traceStore io.Writer, appOpts servertypes.AppOptions) servertypes.Application {
    return app.NewApp(logger, db, traceStore, true, appOpts)
}

func appExport(
    logger log.Logger, db dbm.DB, traceStore io.Writer, height int64,
    forZeroHeight bool, jailAllowedAddrs []string, appOpts servertypes.AppOptions,
) (servertypes.ExportedApp, error) {
    app := app.NewApp(logger, db, traceStore, false, appOpts)
    if height != -1 {
        if err := app.LoadHeight(height); err != nil {
            return servertypes.ExportedApp{}, err
        }
    }
    return app.ExportAppStateAndValidators(forZeroHeight, jailAllowedAddrs, nil)
}

func addModuleInitFlags(startCmd *cobra.Command) {
    crisis.AddModuleInitFlags(startCmd)
}

func initConfigAndDefaultGenesis() *tmcfg.Config {
    cfg := tmcfg.DefaultConfig()
    cfg.Consensus.TimeoutCommit = 5 * time.Second
    return cfg
}
