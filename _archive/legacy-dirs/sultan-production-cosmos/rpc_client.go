package main

import (
    "context"
    "fmt"

    "github.com/cometbft/cometbft/rpc/client/http"
)

func main() {
    c, err := http.New("http://localhost:26657", "/websocket")
    if err != nil {
        panic(err)
    }
    err = c.Start()
    if err != nil {
        panic(err)
    }
    defer c.Stop()

    res, err := c.Status(context.Background())
    if err != nil {
        panic(err)
    }

    fmt.Printf("Node ID: %s\n", res.NodeInfo.NodeID)
    fmt.Printf("Latest Block Height: %d\n", res.SyncInfo.LatestBlockHeight)
}
