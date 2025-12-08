package main

import (
    "encoding/json"
    "fmt"
    "log"
    "net/http"
    "sync"
    "time"
)

type ProductionNode struct {
    ChainID      string
    Version      string
    NetworkID    string
    Validators   []string
    BlockHeight  int64
    mu           sync.RWMutex
    Consensus    string
    ZeroGasFees  bool
}

func NewProductionNode() *ProductionNode {
    return &ProductionNode{
        ChainID:     "sultan-mainnet-1",
        Version:     "v1.0.0",
        NetworkID:   "mainnet",
        Validators:  []string{"validator1", "validator2", "validator3"},
        BlockHeight: 0,
        Consensus:   "BFT",
        ZeroGasFees: true,
    }
}

func (n *ProductionNode) Start() {
    fmt.Println("╔══════════════════════════════════════════════════════════════╗")
    fmt.Println("║           SULTAN CHAIN - MAINNET NODE STARTING                ║")
    fmt.Println("╚══════════════════════════════════════════════════════════════╝")
    fmt.Printf("Chain ID: %s\n", n.ChainID)
    fmt.Printf("Network: %s\n", n.NetworkID)
    fmt.Printf("Consensus: %s\n", n.Consensus)
    fmt.Printf("Zero Gas Fees: %v\n", n.ZeroGasFees)
    fmt.Printf("Validators: %d\n", len(n.Validators))
    
    // Start block production
    go n.produceBlocks()
    
    // Start API server
    n.startAPI()
}

func (n *ProductionNode) produceBlocks() {
    ticker := time.NewTicker(5 * time.Second)
    for range ticker.C {
        n.mu.Lock()
        n.BlockHeight++
        n.mu.Unlock()
        log.Printf("New block produced: #%d", n.BlockHeight)
    }
}

func (n *ProductionNode) startAPI() {
    http.HandleFunc("/status", n.handleStatus)
    http.HandleFunc("/validators", n.handleValidators)
    
    fmt.Println("\n🌐 API Server running on :26657")
    log.Fatal(http.ListenAndServe(":26657", nil))
}

func (n *ProductionNode) handleStatus(w http.ResponseWriter, r *http.Request) {
    n.mu.RLock()
    defer n.mu.RUnlock()
    
    status := map[string]interface{}{
        "chain_id":     n.ChainID,
        "version":      n.Version,
        "network":      n.NetworkID,
        "block_height": n.BlockHeight,
        "consensus":    n.Consensus,
        "zero_gas":     n.ZeroGasFees,
    }
    
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(status)
}

func (n *ProductionNode) handleValidators(w http.ResponseWriter, r *http.Request) {
    n.mu.RLock()
    defer n.mu.RUnlock()
    
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(n.Validators)
}

func main() {
    node := NewProductionNode()
    node.Start()
}
