#!/bin/bash

# Week 1: Add Persistence (CRITICAL)
echo "📦 Adding SQLite persistence..."
cd /workspaces/0xv7/working-chain
cat > persistence.go << 'EOF'
package main

import (
    "database/sql"
    "encoding/json"
    _ "github.com/mattn/go-sqlite3"
)

func (bc *Blockchain) SaveBlock(block Block) error {
    db, err := sql.Open("sqlite3", "./blockchain.db")
    if err != nil {
        return err
    }
    defer db.Close()
    
    blockJSON, _ := json.Marshal(block)
    _, err = db.Exec("INSERT INTO blocks (height, data) VALUES (?, ?)", 
        block.Index, string(blockJSON))
    return err
}

func (bc *Blockchain) LoadBlocks() error {
    db, err := sql.Open("sqlite3", "./blockchain.db")
    if err != nil {
        return err
    }
    defer db.Close()
    
    // Load all blocks from database
    rows, err := db.Query("SELECT data FROM blocks ORDER BY height")
    if err != nil {
        return err
    }
    defer rows.Close()
    
    bc.blocks = []Block{}
    for rows.Next() {
        var blockJSON string
        var block Block
        rows.Scan(&blockJSON)
        json.Unmarshal([]byte(blockJSON), &block)
        bc.blocks = append(bc.blocks, block)
    }
    return nil
}
EOF

# Install SQLite driver
go get github.com/mattn/go-sqlite3
go build -o sultan-chain main.go persistence.go transaction_handler.go
