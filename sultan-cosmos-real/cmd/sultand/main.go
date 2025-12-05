package main

import (
    "fmt"
    "os"

    sdk "github.com/cosmos/cosmos-sdk/types"
    svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
    
    "github.com/sultan-chain/sultan/app"
    "github.com/sultan-chain/sultan/cmd/sultand/cmd"
)

func init() {
    // Ensure SDK config is set before any keeper initialization
    SetupConfig()
}

func SetupConfig() {
    config := sdk.GetConfig()
    config.SetBech32PrefixForAccount(app.AccountAddressPrefix, app.AccountAddressPrefix+"pub")
    config.SetBech32PrefixForValidator(app.AccountAddressPrefix+"valoper", app.AccountAddressPrefix+"valoperpub")
    config.SetBech32PrefixForConsensusNode(app.AccountAddressPrefix+"valcons", app.AccountAddressPrefix+"valconspub")
    config.Seal()
}

func main() {
    rootCmd := cmd.NewRootCmd()
    if err := svrcmd.Execute(rootCmd, "", app.DefaultNodeHome); err != nil {
        fmt.Fprintln(os.Stderr, err)
        os.Exit(1)
    }
}
