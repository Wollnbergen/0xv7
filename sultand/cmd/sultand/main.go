package main

import (
	"os"
	
	"cosmossdk.io/log"
	svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
	
	"github.com/wollnbergen/sultand/app"
	"github.com/wollnbergen/sultand/cmd/sultand/cmd"
)

func main() {
	rootCmd := cmd.NewRootCmd()
	
	if err := svrcmd.Execute(rootCmd, "", app.DefaultNodeHome); err != nil {
		log.NewLogger(os.Stdout).Error("failure when running app", "err", err)
		os.Exit(1)
	}
}
