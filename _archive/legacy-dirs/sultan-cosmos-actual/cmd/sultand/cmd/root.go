package cmd

import (
    "os"
    
    "github.com/cosmos/cosmos-sdk/client"
    "github.com/cosmos/cosmos-sdk/client/config"
    "github.com/cosmos/cosmos-sdk/client/debug"
    "github.com/cosmos/cosmos-sdk/client/flags"
    "github.com/cosmos/cosmos-sdk/client/keys"
    "github.com/cosmos/cosmos-sdk/client/rpc"
    "github.com/cosmos/cosmos-sdk/server"
    serverconfig "github.com/cosmos/cosmos-sdk/server/config"
    servertypes "github.com/cosmos/cosmos-sdk/server/types"
    "github.com/cosmos/cosmos-sdk/types/module"
    authcmd "github.com/cosmos/cosmos-sdk/x/auth/client/cli"
    bankcmd "github.com/cosmos/cosmos-sdk/x/bank/client/cli"
    "github.com/cosmos/cosmos-sdk/x/crisis"
    genutilcli "github.com/cosmos/cosmos-sdk/x/genutil/client/cli"
    "github.com/spf13/cobra"
    
    "github.com/sultan/sultan-cosmos-actual/app"
)

// NewRootCmd creates the root command for sultand
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
        Short: "Sultan Chain - Real Cosmos SDK Implementation",
        PersistentPreRunE: func(cmd *cobra.Command, _ []string) error {
            initClientCtx, err := client.ReadPersistentCommandFlags(initClientCtx, cmd.Flags())
            if err != nil {
                return err
            }
            
            initClientCtx, err = config.ReadFromClientConfig(initClientCtx)
            if err != nil {
                return err
            }
            
            if err := client.SetCmdClientContext(cmd, initClientCtx); err != nil {
                return err
            }
            
            customAppTemplate, customAppConfig := initAppConfig()
            customTMConfig := initTendermintConfig()
            
            return server.InterceptConfigsPreRunHandler(
                cmd, customAppTemplate, customAppConfig, customTMConfig,
            )
        },
    }
    
    initRootCmd(rootCmd, encodingConfig)
    
    return rootCmd
}

func initRootCmd(rootCmd *cobra.Command, encodingConfig params.EncodingConfig) {
    rootCmd.AddCommand(
        genutilcli.InitCmd(app.ModuleBasics, app.DefaultNodeHome),
        debug.Cmd(),
        config.Cmd(),
        server.NewStartCmd(app.NewSultanApp, app.DefaultNodeHome),
        server.StatusCommand(),
        keys.Commands(app.DefaultNodeHome),
        rpc.StatusCommand(),
        authcmd.QueryTxCmd(),
        authcmd.GetAccountCmd(),
        bankcmd.GetBalanceCmd(),
        bankcmd.SendTxCmd(),
    )
}
