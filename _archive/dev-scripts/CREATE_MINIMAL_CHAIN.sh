#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         CREATING MINIMAL ZERO-GAS BLOCKCHAIN                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Building a lightweight blockchain that actually works..."
echo ""

# Create minimal structure
MINIMAL_DIR="/workspaces/0xv7/minimal-chain"
rm -rf $MINIMAL_DIR
mkdir -p $MINIMAL_DIR
cd $MINIMAL_DIR

# Step 1: Create a simple Go blockchain
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📝 [1/4] Creating minimal blockchain..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cat > main.go << 'GOCODE'
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
GOCODE

echo "✅ Minimal blockchain created"

# Step 2: Create go.mod
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 [2/4] Setting up Go module..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cat > go.mod << 'GOMOD'
module minimal-chain

go 1.21
GOMOD

echo "✅ Go module configured"

# Step 3: Build the blockchain
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔨 [3/4] Building blockchain..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

go build -o minimal-chain main.go

if [ -f "./minimal-chain" ]; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed"
    exit 1
fi

# Step 4: Create test dashboard
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎨 [4/4] Creating dashboard..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cat > /workspaces/0xv7/public/minimal-dashboard.html << 'HTML'
<!DOCTYPE html>
<html>
<head>
    <title>Minimal Zero-Gas Chain</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 20px;
        }
        .container {
            max-width: 800px;
            margin: 0 auto;
        }
        .block {
            background: rgba(255,255,255,0.1);
            padding: 15px;
            margin: 10px 0;
            border-radius: 10px;
        }
        button {
            background: #00ff88;
            color: black;
            border: none;
            padding: 10px 20px;
            border-radius: 5px;
            cursor: pointer;
            font-size: 16px;
        }
        input {
            padding: 10px;
            width: 300px;
            border-radius: 5px;
            border: none;
            margin-right: 10px;
        }
        .zero-gas {
            color: #00ff88;
            font-size: 24px;
            font-weight: bold;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Minimal Zero-Gas Blockchain</h1>
        <p class="zero-gas">Gas Fees: $0.00 FOREVER!</p>
        
        <div>
            <input type="text" id="dataInput" placeholder="Enter data for new block">
            <button onclick="addBlock()">Add Block (Zero Gas!)</button>
        </div>
        
        <h2>Blockchain:</h2>
        <div id="blocks"></div>
    </div>
    
    <script>
        async function loadBlocks() {
            const response = await fetch('http://localhost:8080/blocks');
            const blocks = await response.json();
            const container = document.getElementById('blocks');
            container.innerHTML = blocks.map(block => `
                <div class="block">
                    <strong>Block #${block.index}</strong><br>
                    Data: ${block.data}<br>
                    Hash: ${block.hash.substring(0, 20)}...<br>
                    Gas Fee: $${block.gas_fee}
                </div>
            `).join('');
        }
        
        async function addBlock() {
            const data = document.getElementById('dataInput').value;
            if (!data) return;
            
            await fetch('http://localhost:8080/write', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({data: data})
            });
            
            document.getElementById('dataInput').value = '';
            loadBlocks();
        }
        
        setInterval(loadBlocks, 2000);
        loadBlocks();
    </script>
</body>
</html>
HTML

echo "✅ Dashboard created"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ MINIMAL CHAIN READY!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "To run:"
echo "  cd $MINIMAL_DIR"
echo "  ./minimal-chain"
echo ""
echo "Then open dashboard:"
echo "  $BROWSER http://localhost:3000/minimal-dashboard.html"

