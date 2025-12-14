package main

import (
    "crypto/sha256"
    "encoding/hex"
    "encoding/json"
    "fmt"
    "log"
    "net/http"
    "strconv"
    "sync"
    "time"
)

type Transaction struct {
    From     string    `json:"from"`
    To       string    `json:"to"`
    Amount   float64   `json:"amount"`
    GasFee   float64   `json:"gas_fee"`
    Data     string    `json:"data"`
    Time     time.Time `json:"time"`
}

type Block struct {
    Index        int           `json:"index"`
    Timestamp    string        `json:"timestamp"`
    Transactions []Transaction `json:"transactions"`
    Hash         string        `json:"hash"`
    PrevHash     string        `json:"prev_hash"`
    Nonce        int          `json:"nonce"`
    Validator    string        `json:"validator"`
}

type Blockchain struct {
    blocks              []Block
    pendingTransactions []Transaction
    mu                  sync.RWMutex
    validators          []string
}

func (bc *Blockchain) calculateHash(block Block) string {
    record := strconv.Itoa(block.Index) + block.Timestamp + fmt.Sprintf("%v", block.Transactions) + block.PrevHash + strconv.Itoa(block.Nonce)
    h := sha256.New()
    h.Write([]byte(record))
    hashed := h.Sum(nil)
    return hex.EncodeToString(hashed)
}

func (bc *Blockchain) mineBlock() Block {
    bc.mu.Lock()
    defer bc.mu.Unlock()
    
    prevBlock := bc.blocks[len(bc.blocks)-1]
    
    newBlock := Block{
        Index:        prevBlock.Index + 1,
        Timestamp:    time.Now().String(),
        Transactions: bc.pendingTransactions,
        PrevHash:     prevBlock.Hash,
        Nonce:        0,
        Validator:    bc.validators[prevBlock.Index%len(bc.validators)],
    }
    
    // Simple Proof of Work
    for i := 0; ; i++ {
        newBlock.Nonce = i
        newBlock.Hash = bc.calculateHash(newBlock)
        if newBlock.Hash[:2] == "00" {
            break
        }
    }
    
    bc.blocks = append(bc.blocks, newBlock)
    bc.pendingTransactions = []Transaction{}
    
    return newBlock
}

func (bc *Blockchain) addTransaction(tx Transaction) {
    bc.mu.Lock()
    defer bc.mu.Unlock()
    
    // Zero gas fees!
    tx.GasFee = 0
    tx.Time = time.Now()
    bc.pendingTransactions = append(bc.pendingTransactions, tx)
}

func (bc *Blockchain) getStatus() map[string]interface{} {
    bc.mu.RLock()
    defer bc.mu.RUnlock()
    
    return map[string]interface{}{
        "chain_id":      "sultan-mainnet-1",
        "block_height":  len(bc.blocks) - 1,
        "latest_hash":   bc.blocks[len(bc.blocks)-1].Hash,
        "pending_txs":   len(bc.pendingTransactions),
        "zero_gas":      true,
        "validators":    bc.validators,
        "consensus":     "SimplePoW",
        "network":       "mainnet",
        "version":       "v1.0.0",
    }
}

func main() {
    // Initialize blockchain with genesis block
    blockchain := &Blockchain{
        validators: []string{"validator-1", "validator-2", "validator-3"},
        blocks: []Block{
            {
                Index:        0,
                Timestamp:    time.Now().String(),
                Transactions: []Transaction{},
                Hash:         "00genesis",
                PrevHash:     "",
                Nonce:        0,
                Validator:    "genesis",
            },
        },
    }
    
    // Start consensus - mine blocks every 5 seconds if there are transactions
    go func() {
        for {
            time.Sleep(5 * time.Second)
            blockchain.mu.RLock()
            hasTxs := len(blockchain.pendingTransactions) > 0
            blockchain.mu.RUnlock()
            
            if hasTxs {
                block := blockchain.mineBlock()
                log.Printf("⛏️ Block #%d mined by %s | Transactions: %d | Hash: %s", 
                    block.Index, block.Validator, len(block.Transactions), block.Hash[:8])
            }
        }
    }()
    
    // HTTP API
    http.HandleFunc("/status", func(w http.ResponseWriter, r *http.Request) {
    http.HandleFunc("/api/transaction", blockchain.handleTransaction)
    http.HandleFunc("/api/submit", blockchain.handleTransaction)
    http.HandleFunc("/transaction", blockchain.handleTransaction)
        w.Header().Set("Content-Type", "application/json")
        json.NewEncoder(w).Encode(blockchain.getStatus())
    })
    
    http.HandleFunc("/blocks", func(w http.ResponseWriter, r *http.Request) {
        blockchain.mu.RLock()
        defer blockchain.mu.RUnlock()
        w.Header().Set("Content-Type", "application/json")
        json.NewEncoder(w).Encode(blockchain.blocks)
    })
    
    http.HandleFunc("/tx", func(w http.ResponseWriter, r *http.Request) {
        if r.Method == "POST" {
            var tx Transaction
            json.NewDecoder(r.Body).Decode(&tx)
            blockchain.addTransaction(tx)
            
            w.Header().Set("Content-Type", "application/json")
            json.NewEncoder(w).Encode(map[string]interface{}{
                "success": true,
                "message": "Transaction added with ZERO gas fees!",
                "tx":      tx,
                "gas_fee": 0,
            })
        } else if r.Method == "GET" {
            blockchain.mu.RLock()
            defer blockchain.mu.RUnlock()
            w.Header().Set("Content-Type", "application/json")
            json.NewEncoder(w).Encode(blockchain.pendingTransactions)
        }
    })
    
    fmt.Println("╔══════════════════════════════════════════════════════════════╗")
    fmt.Println("║          SULTAN CHAIN - ZERO GAS BLOCKCHAIN RUNNING           ║")
    fmt.Println("╚══════════════════════════════════════════════════════════════╝")
    fmt.Println("")
    fmt.Printf("🔗 Chain ID: sultan-mainnet-1\n")
    fmt.Printf("💰 Gas Fees: ZERO (Always Free!)\n")
    fmt.Printf("⚡ Consensus: SimplePoW with 3 validators\n")
    fmt.Printf("🌐 API: http://localhost:8080\n")
    fmt.Printf("📊 Status: http://localhost:8080/status\n")
    fmt.Printf("📦 Blocks: http://localhost:8080/blocks\n")
    fmt.Printf("💸 Send TX: POST http://localhost:8080/tx\n\n")
    
    log.Fatal(http.ListenAndServe(":8080", nil))
}
