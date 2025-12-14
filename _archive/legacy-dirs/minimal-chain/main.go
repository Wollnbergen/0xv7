package main

import (
    "crypto/sha256"
    "encoding/hex"
    "encoding/json"
    "fmt"
    "log"
    "net/http"
    "time"
)

type Block struct {
    Index     int       `json:"index"`
    Timestamp time.Time `json:"timestamp"`
    Data      string    `json:"data"`
    Hash      string    `json:"hash"`
    PrevHash  string    `json:"prevhash"`
    GasFee    float64   `json:"gas_fee"` // Always 0!
}

type Blockchain struct {
    blocks []Block
}

var chain Blockchain

func (b Block) calculateHash() string {
    record := fmt.Sprintf("%d%s%s%s", b.Index, b.Timestamp, b.Data, b.PrevHash)
    h := sha256.New()
    h.Write([]byte(record))
    hashed := h.Sum(nil)
    return hex.EncodeToString(hashed)
}

func generateBlock(oldBlock Block, data string) Block {
    var newBlock Block
    newBlock.Index = oldBlock.Index + 1
    newBlock.Timestamp = time.Now()
    newBlock.Data = data
    newBlock.PrevHash = oldBlock.Hash
    newBlock.GasFee = 0.0 // Zero gas fees!
    newBlock.Hash = newBlock.calculateHash()
    return newBlock
}

func handleGetBlockchain(w http.ResponseWriter, r *http.Request) {
    w.Header().Set("Access-Control-Allow-Origin", "*")
    json.NewEncoder(w).Encode(chain.blocks)
}

func handleWriteBlock(w http.ResponseWriter, r *http.Request) {
    w.Header().Set("Access-Control-Allow-Origin", "*")
    var data struct {
        Data string `json:"data"`
    }
    
    if err := json.NewDecoder(r.Body).Decode(&data); err != nil {
        http.Error(w, err.Error(), http.StatusBadRequest)
        return
    }
    
    newBlock := generateBlock(chain.blocks[len(chain.blocks)-1], data.Data)
    chain.blocks = append(chain.blocks, newBlock)
    
    json.NewEncoder(w).Encode(newBlock)
}

func handleStatus(w http.ResponseWriter, r *http.Request) {
    w.Header().Set("Access-Control-Allow-Origin", "*")
    status := map[string]interface{}{
        "chain_id": "minimal-1",
        "height": len(chain.blocks),
        "gas_fee": 0.0,
        "status": "running",
    }
    json.NewEncoder(w).Encode(status)
}

func main() {
    // Genesis block
    genesis := Block{
        Index:     0,
        Timestamp: time.Now(),
        Data:      "Genesis Block - Zero Gas Fees Forever!",
        Hash:      "",
        PrevHash:  "",
        GasFee:    0.0,
    }
    genesis.Hash = genesis.calculateHash()
    chain.blocks = append(chain.blocks, genesis)
    
    http.HandleFunc("/blocks", handleGetBlockchain)
    http.HandleFunc("/write", handleWriteBlock)
    http.HandleFunc("/status", handleStatus)
    
    fmt.Println("╔══════════════════════════════════════════════════════════════╗")
    fmt.Println("║         MINIMAL ZERO-GAS BLOCKCHAIN RUNNING!                  ║")
    fmt.Println("╚══════════════════════════════════════════════════════════════╝")
    fmt.Println("")
    fmt.Println("🚀 Chain ID: minimal-1")
    fmt.Println("💸 Gas Fees: ZERO")
    fmt.Println("🌐 API: http://localhost:8080")
    fmt.Println("")
    fmt.Println("Endpoints:")
    fmt.Println("  GET  /blocks - View all blocks")
    fmt.Println("  POST /write  - Add new block (JSON: {\"data\":\"your data\"})")
    fmt.Println("  GET  /status - Chain status")
    
    log.Fatal(http.ListenAndServe(":8080", nil))
}
