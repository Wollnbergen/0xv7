package main

import (
    "context"
    "fmt"
    "github.com/libp2p/go-libp2p"
    "github.com/libp2p/go-libp2p/core/host"
)

func main() {
    // Create a new libp2p Host
    host, err := libp2p.New(
        libp2p.ListenAddrStrings("/ip4/0.0.0.0/tcp/0"),
    )
    if err != nil {
        panic(err)
    }
    defer host.Close()
    
    fmt.Println("LibP2P node started!")
    fmt.Println("Node ID:", host.ID())
    fmt.Println("Addresses:", host.Addrs())
    
    // Keep running
    select {}
}
