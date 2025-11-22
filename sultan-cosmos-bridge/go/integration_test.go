package main

import (
	"encoding/json"
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/wollnbergen/sultan-cosmos-bridge/bridge"
	"github.com/wollnbergen/sultan-cosmos-bridge/types"
)

// TestFullBlockchainFlow tests the complete lifecycle
func TestFullBlockchainFlow(t *testing.T) {
	err := bridge.Initialize()
	require.NoError(t, err)
	defer bridge.Shutdown()

	bc, err := bridge.NewBlockchain()
	require.NoError(t, err)
	defer bc.Destroy()

	// Initialize genesis accounts
	accounts := []struct {
		address string
		balance uint64
	}{
		{"alice", 1000000},
		{"bob", 500000},
		{"charlie", 250000},
		{"validator1", 100000},
	}

	for _, acc := range accounts {
		err := bc.InitAccount(acc.address, acc.balance)
		require.NoError(t, err, "Failed to init account %s", acc.address)
	}

	// Verify genesis balances
	for _, acc := range accounts {
		balance, err := bc.GetBalance(acc.address)
		require.NoError(t, err)
		assert.Equal(t, acc.balance, balance, "Wrong balance for %s", acc.address)
	}

	// Submit transactions
	transactions := []types.Transaction{
		{From: "alice", To: "bob", Amount: 1000, Nonce: 1, Timestamp: uint64(time.Now().Unix())},
		{From: "bob", To: "charlie", Amount: 500, Nonce: 1, Timestamp: uint64(time.Now().Unix())},
		{From: "alice", To: "charlie", Amount: 2000, Nonce: 2, Timestamp: uint64(time.Now().Unix())},
	}

	for _, tx := range transactions {
		err := bc.AddTransaction(tx)
		require.NoError(t, err, "Failed to add transaction %v", tx)
	}

	// Create block
	err = bc.CreateBlock("validator1")
	require.NoError(t, err)

	// Verify block height
	height, err := bc.Height()
	require.NoError(t, err)
	assert.Equal(t, uint64(1), height)

	// Get latest hash
	hash, err := bc.LatestHash()
	require.NoError(t, err)
	assert.NotEmpty(t, hash)
	fmt.Printf("Block 1 hash: %s\n", hash)
}

// TestABCIFullProtocol tests the complete ABCI protocol flow
func TestABCIFullProtocol(t *testing.T) {
	err := bridge.Initialize()
	require.NoError(t, err)
	defer bridge.Shutdown()

	bc, err := bridge.NewBlockchain()
	require.NoError(t, err)
	defer bc.Destroy()

	// Step 1: Info request
	infoReq := types.ABCIRequest{Type: "Info"}
	infoResp, err := bc.ProcessABCI(infoReq)
	require.NoError(t, err)
	assert.Equal(t, "Info", infoResp.Type)
	assert.Equal(t, uint64(0), infoResp.Height)

	// Step 2: InitChain
	initReq := types.ABCIRequest{
		Type: "InitChain",
		GenesisAccounts: []types.GenesisAccount{
			{Address: "alice", Balance: 1000000},
			{Address: "validator1", Balance: 100000},
		},
	}
	initResp, err := bc.ProcessABCI(initReq)
	require.NoError(t, err)
	assert.Equal(t, "InitChainOk", initResp.Type)

	// Step 3: BeginBlock
	beginReq := types.ABCIRequest{
		Type:     "BeginBlock",
		Height:   1,
		Proposer: "validator1",
	}
	beginResp, err := bc.ProcessABCI(beginReq)
	require.NoError(t, err)
	assert.Equal(t, "BeginBlockOk", beginResp.Type)

	// Step 4: DeliverTx
	tx := types.Transaction{
		From:      "alice",
		To:        "validator1",
		Amount:    1000,
		Nonce:     1,
		Timestamp: uint64(time.Now().Unix()),
	}
	txData, _ := json.Marshal(tx)
	deliverReq := types.ABCIRequest{
		Type:   "DeliverTx",
		TxData: txData,
	}
	deliverResp, err := bc.ProcessABCI(deliverReq)
	require.NoError(t, err)
	assert.Equal(t, "DeliverTx", deliverResp.Type)
	if deliverResp.Code != 0 {
		t.Logf("DeliverTx failed with code %d: %s", deliverResp.Code, deliverResp.Log)
	}
	assert.Equal(t, uint32(0), deliverResp.Code)

	// Step 5: EndBlock
	endReq := types.ABCIRequest{
		Type:   "EndBlock",
		Height: 1,
	}
	endResp, err := bc.ProcessABCI(endReq)
	require.NoError(t, err)
	assert.Equal(t, "EndBlockOk", endResp.Type)

	// Step 6: Commit
	commitReq := types.ABCIRequest{Type: "Commit"}
	commitResp, err := bc.ProcessABCI(commitReq)
	require.NoError(t, err)
	assert.Equal(t, "Commit", commitResp.Type)
	assert.NotEmpty(t, commitResp.Data)

	// Step 7: Query balance
	queryReq := types.ABCIRequest{
		Type: "Query",
		Path: "/balance/alice",
	}
	queryResp, err := bc.ProcessABCI(queryReq)
	require.NoError(t, err)
	assert.Equal(t, "Query", queryResp.Type)
	if queryResp.Code != 0 {
		t.Logf("Query failed with code %d: %s", queryResp.Code, queryResp.Log)
	}
	assert.Equal(t, uint32(0), queryResp.Code)
}

// TestConsensusWithMultipleValidators tests validator management and proposer selection
func TestConsensusWithMultipleValidators(t *testing.T) {
	err := bridge.Initialize()
	require.NoError(t, err)
	defer bridge.Shutdown()

	consensus, err := bridge.NewConsensusEngine()
	require.NoError(t, err)

	// Add validators with different stakes
	validators := []struct {
		address string
		stake   uint64
	}{
		{"validator1", 100000},
		{"validator2", 50000},
		{"validator3", 25000},
	}

	for _, v := range validators {
		err := consensus.AddValidator(v.address, v.stake)
		require.NoError(t, err)
	}

	// Select proposer multiple times and verify it's always a registered validator
	proposerCounts := make(map[string]int)
	iterations := 100

	for i := 0; i < iterations; i++ {
		proposer, err := consensus.SelectProposer()
		require.NoError(t, err)
		require.NotEmpty(t, proposer)

		// Verify proposer is a registered validator
		isValid := false
		for _, v := range validators {
			if v.address == proposer {
				isValid = true
				break
			}
		}
		assert.True(t, isValid, "Selected proposer %s is not a registered validator", proposer)

		proposerCounts[proposer]++
	}

	// Verify all validators were selected at least once (probabilistic)
	// With weighted selection, validator1 should be selected most often
	fmt.Printf("Proposer selection distribution over %d iterations:\n", iterations)
	for address, count := range proposerCounts {
		fmt.Printf("  %s: %d times (%.1f%%)\n", address, count, float64(count)/float64(iterations)*100)
	}

	assert.Greater(t, proposerCounts["validator1"], proposerCounts["validator2"],
		"validator1 (higher stake) should be selected more often")
}

// TestStressTest - Submit many transactions and create multiple blocks
func TestStressTest(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping stress test in short mode")
	}

	err := bridge.Initialize()
	require.NoError(t, err)
	defer bridge.Shutdown()

	bc, err := bridge.NewBlockchain()
	require.NoError(t, err)
	defer bc.Destroy()

	// Setup accounts
	bc.InitAccount("alice", 10000000)
	bc.InitAccount("bob", 5000000)
	bc.InitAccount("validator1", 100000)

	// Submit many transactions
	txCount := 100
	successCount := 0

	start := time.Now()
	for i := 1; i <= txCount; i++ {
		tx := types.Transaction{
			From:      "alice",
			To:        "bob",
			Amount:    100,
			Nonce:     uint64(i),
			Timestamp: uint64(time.Now().Unix()),
		}

		err := bc.AddTransaction(tx)
		if err == nil {
			successCount++
		}
	}
	elapsed := time.Since(start)

	fmt.Printf("Submitted %d/%d transactions in %v (%.0f tx/sec)\n",
		successCount, txCount, elapsed, float64(successCount)/elapsed.Seconds())

	// Create multiple blocks
	blockCount := 10
	for i := 0; i < blockCount; i++ {
		err := bc.CreateBlock("validator1")
		require.NoError(t, err)
	}

	// Verify final height
	height, err := bc.Height()
	require.NoError(t, err)
	assert.Equal(t, uint64(blockCount), height)

	fmt.Printf("Created %d blocks successfully\n", blockCount)
}

// BenchmarkFullBlockProduction benchmarks the complete block production cycle
func BenchmarkFullBlockProduction(b *testing.B) {
	bridge.Initialize()
	defer bridge.Shutdown()

	bc, _ := bridge.NewBlockchain()
	defer bc.Destroy()

	bc.InitAccount("alice", 10000000)
	bc.InitAccount("bob", 5000000)
	bc.InitAccount("validator1", 100000)

	// Pre-submit transactions
	for i := 1; i <= 10; i++ {
		tx := types.Transaction{
			From:      "alice",
			To:        "bob",
			Amount:    100,
			Nonce:     uint64(i),
			Timestamp: uint64(time.Now().Unix()),
		}
		bc.AddTransaction(tx)
	}

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		bc.CreateBlock("validator1")
	}
}
