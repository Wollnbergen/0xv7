package main

import (
    "os"
    "sultan/app"
)

func main() {
    if err := Execute(); err != nil {
        os.Exit(1)
    }
}

func Execute() error {
    // Simplified for now - will be expanded
    return nil
}
