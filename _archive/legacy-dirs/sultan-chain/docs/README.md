### Step 1: Install Go and Ignite CLI

First, ensure you have Go 1.20+ installed. You can check your Go version with:

```bash
go version
```

If Go is not installed, you can install it using:

```bash
sudo apt update
sudo apt install golang-go
```

Next, install Ignite CLI:

```bash
curl https://get.ignite.com/cli! | bash
```

### Step 2: Scaffold a New Chain Project

Now, scaffold a new chain project named `sultan`:

```bash
ignite scaffold chain sultan --no-module
cd sultan
```

This command creates a new Go module with the necessary dependencies, including the Cosmos SDK.

### Step 3: Add a Custom Module

For demonstration, let's add a simple module called `blog`:

```bash
ignite scaffold module blog
ignite scaffold message create-post --module blog title body
```

### Step 4: Import Actual Cosmos SDK Packages

In the generated code, you will see imports like these in your module's handler file (e.g., `x/blog/keeper/msg_server_create_post.go`):

```go
package keeper

import (
    "context"

    "cosmossdk.io/errors" // For error handling
    "github.com/cosmos/cosmos-sdk/types" // Core types like Context, Msg
    sdk "github.com/cosmos/cosmos-sdk/types" // Alias for convenience
    "github.com/cosmos/cosmos-sdk/x/auth" // For authentication if needed
    "sultan/x/blog/types" // Your module's types
)
```

### Step 5: Connect to Real Tendermint (CometBFT)

Add the CometBFT RPC client dependency:

```bash
go get github.com/cometbft/cometbft/rpc/client/http
```

You can create a new file (e.g., `tendermint_client.go`) to connect to a Tendermint node:

```go
package main

import (
    "context"
    "fmt"

    "github.com/cometbft/cometbft/rpc/client/http"
)

func connectToTendermint() {
    c, err := http.New("http://localhost:26657", "/websocket")
    if err != nil {
        panic(err)
    }

    err = c.Start()
    if err != nil {
        panic(err)
    }
    defer c.Stop()

    ctx := context.Background()
    res, err := c.Status(ctx)
    if err != nil {
        panic(err)
    }

    fmt.Printf("Node ID: %s\n", res.NodeInfo.NodeID)
    fmt.Printf("Latest Block Height: %d\n", res.SyncInfo.LatestBlockHeight)
    fmt.Printf("Chain ID: %s\n", res.NodeInfo.Network)
}
```

### Step 6: Implement P2P Networking with Libp2p

Add the Libp2p dependency:

```bash
go get github.com/libp2p/go-libp2p
```

Create a new file (e.g., `p2p_node.go`) for the P2P networking setup:

```go
package main

import (
    "context"
    "fmt"
    "os"
    "os/signal"
    "syscall"

    "github.com/libp2p/go-libp2p"
    "github.com/libp2p/go-libp2p/core/peer"
    "github.com/libp2p/go-libp2p/p2p/protocol/ping"
    ma "github.com/multiformats/go-multiaddr"
)

func startP2PNode() {
    node, err := libp2p.New()
    if err != nil {
        panic(err)
    }

    pingService := &ping.PingService{Host: node}
    node.SetStreamHandler(ping.ID, pingService.PingHandler)

    peerInfo := peer.AddrInfo{
        ID:    node.ID(),
        Addrs: node.Addrs(),
    }
    addrs, err := peer.AddrInfoToP2pAddrs(&peerInfo)
    if err != nil {
        panic(err)
    }
    fmt.Println("libp2p node address:", addrs[0])

    ch := make(chan os.Signal, 1)
    signal.Notify(ch, syscall.SIGINT, syscall.SIGTERM)
    <-ch
    fmt.Println("Shutting down...")
}
```

### Step 7: Build and Run the Project

Now, build your project:

```bash
ignite chain build
```

To run your chain, use:

```bash
./sultand start
```

### Step 8: Test the P2P Networking

You can run the P2P node in a separate terminal:

```bash
go run p2p_node.go
```

### Summary

You have now created a new Cosmos SDK project with actual Tendermint consensus and P2P networking integration. You can initialize and run your blockchain, connect to a Tendermint node, and set up a basic P2P network.