package main

import (
    "encoding/json"
    "net/http"
    "time"
)

// TransactionRequest represents an incoming transaction request
type TransactionRequest struct {
    From    string  `json:"from"`
    To      string  `json:"to"`
    Amount  float64 `json:"amount"`
    Message string  `json:"message,omitempty"`
}

// handleTransaction processes new transactions
func (bc *Blockchain) handleTransaction(w http.ResponseWriter, r *http.Request) {
    if r.Method != "POST" {
        http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
        return
    }

    var req TransactionRequest
    if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
        http.Error(w, err.Error(), http.StatusBadRequest)
        return
    }

    // Create transaction with zero gas fee
    tx := Transaction{
        From:   req.From,
        To:     req.To,
        Amount: req.Amount,
        GasFee: 0, // Zero gas fees!
        Data:   req.Message,
        Time:   time.Now(),
    }

    // Add to pending transactions
    bc.mu.Lock()
    bc.pendingTransactions = append(bc.pendingTransactions, tx)
    bc.mu.Unlock()

    // Return success response
    response := map[string]interface{}{
        "success": true,
        "message": "Transaction submitted successfully",
        "tx": map[string]interface{}{
            "from":    tx.From,
            "to":      tx.To,
            "amount":  tx.Amount,
            "gas_fee": tx.GasFee,
            "data":    tx.Data,
        },
    }

    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(response)
}
