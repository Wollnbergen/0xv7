package main

import (
    "context"
    "fmt"
    rpchttp "github.com/cometbft/cometbft/rpc/client/http"
)

func main() {
    // Connect to a CometBFT node
    client, err := rpchttp.New("http://localhost:26657", "/websocket")
    if err != nil {
        fmt.Println("Note: This requires a running CometBFT node")
        fmt.Println("Error:", err)
        return
    }
    
    err = client.Start()
    if err != nil {
        panic(err)
    }
    defer client.Stop()
    
    // Get node status
    status, err := client.Status(context.Background())
    if err != nil {
        panic(err)
    }
    
    fmt.Printf("Connected to node: %s\n", status.NodeInfo.Moniker)
    fmt.Printf("Latest block: %d\n", status.SyncInfo.LatestBlockHeight)
}
