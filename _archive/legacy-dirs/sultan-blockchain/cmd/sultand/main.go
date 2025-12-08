package main

import (
    "fmt"
    "os"
    
    "cosmossdk.io/log"
    dbm "github.com/cometbft/cometbft-db"
    "sultan/app"
    "sultan/cmd/sultand/cmd"
)

func main() {
    rootCmd := cmd.NewRootCmd()
    
    if err := rootCmd.Execute(); err != nil {
        fmt.Fprintf(os.Stderr, "Error: %v\n", err)
        os.Exit(1)
    }
}
