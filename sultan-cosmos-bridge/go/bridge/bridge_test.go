package bridge

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/wollnbergen/sultan-cosmos-bridge/types"
)

func TestBridgeInitialization(t *testing.T) {
	err := Initialize()
	assert.NoError(t, err, "Bridge initialization should succeed")
	
	defer func() {
		err := Shutdown()
		assert.NoError(t, err, "Bridge shutdown should succeed")
	}()
}

func TestBlockchainLifecycle(t *testing.T) {
	err := Initialize()
	require.NoError(t, err)
	defer Shutdown()
	
	// Create blockchain
	bc, err := NewBlockchain()
	require.NoError(t, err, "Should create blockchain")
	require.NotNil(t, bc, "Blockchain should not be nil")
	
	// Check initial height
	height, err := bc.Height()
	require.NoError(t, err)
	assert.Equal(t, uint64(0), height, "Initial height should be 0")
	
	// Get latest hash
	hash, err := bc.LatestHash()
	require.NoError(t, err)
	assert.NotEmpty(t, hash, "Genesis hash should not be empty")
	
	// Destroy blockchain
	err = bc.Destroy()
	assert.NoError(t, err, "Should destroy blockchain")
	
	// Verify destroyed
	_, err = bc.Height()
	assert.Error(t, err, "Should error after destroy")
}

func TestGenesisAccounts(t *testing.T) {
	err := Initialize()
	require.NoError(t, err)
	defer Shutdown()
	
	bc, err := NewBlockchain()
	require.NoError(t, err)
	defer bc.Destroy()
	
	// Initialize genesis accounts
	err = bc.InitAccount("alice", 1000000)
	require.NoError(t, err, "Should initialize alice account")
	
	err = bc.InitAccount("bob", 500000)
	require.NoError(t, err, "Should initialize bob account")
	
	// Check balances
	aliceBalance, err := bc.GetBalance("alice")
	require.NoError(t, err)
	assert.Equal(t, uint64(1000000), aliceBalance, "Alice balance should be 1000000")
	
	bobBalance, err := bc.GetBalance("bob")
	require.NoError(t, err)
	assert.Equal(t, uint64(500000), bobBalance, "Bob balance should be 500000")
	
	// Check non-existent account
	charlieBalance, err := bc.GetBalance("charlie")
	require.NoError(t, err)
	assert.Equal(t, uint64(0), charlieBalance, "Charlie balance should be 0")
}

func TestTransactionSubmission(t *testing.T) {
	err := Initialize()
	require.NoError(t, err)
	defer Shutdown()
	
	bc, err := NewBlockchain()
	require.NoError(t, err)
	defer bc.Destroy()
	
	// Setup accounts
	err = bc.InitAccount("alice", 1000000)
	require.NoError(t, err)
	err = bc.InitAccount("bob", 500000)
	require.NoError(t, err)
	
	// Create transaction
	tx := types.Transaction{
		From:      "alice",
		To:        "bob",
		Amount:    1000,
		GasFee:    0,
		Timestamp: uint64(time.Now().Unix()),
		Nonce:     1,
	}
	
	// Add transaction
	err = bc.AddTransaction(tx)
	require.NoError(t, err, "Should add valid transaction")
	
	// Verify transaction was rejected if invalid
	invalidTx := types.Transaction{
		From:      "alice",
		To:        "bob",
		Amount:    0, // Invalid: zero amount
		GasFee:    0,
		Timestamp: uint64(time.Now().Unix()),
		Nonce:     2,
	}
	
	err = bc.AddTransaction(invalidTx)
	assert.Error(t, err, "Should reject zero-amount transaction")
}

func TestBlockProduction(t *testing.T) {
	err := Initialize()
	require.NoError(t, err)
	defer Shutdown()
	
	bc, err := NewBlockchain()
	require.NoError(t, err)
	defer bc.Destroy()
	
	// Setup genesis
	err = bc.InitAccount("validator1", 100000)
	require.NoError(t, err)
	
	// Get initial height
	height1, err := bc.Height()
	require.NoError(t, err)
	assert.Equal(t, uint64(0), height1)
	
	// Create block
	err = bc.CreateBlock("validator1")
	require.NoError(t, err, "Should create block")
	
	// Verify height increased
	height2, err := bc.Height()
	require.NoError(t, err)
	assert.Equal(t, uint64(1), height2, "Height should increase after block")
}

func TestConsensusEngine(t *testing.T) {
	err := Initialize()
	require.NoError(t, err)
	defer Shutdown()
	
	// Create consensus engine
	consensus, err := NewConsensusEngine()
	require.NoError(t, err, "Should create consensus engine")
	require.NotNil(t, consensus, "Consensus engine should not be nil")
	
	// Add validators
	err = consensus.AddValidator("validator1", 100000)
	require.NoError(t, err, "Should add validator1")
	
	err = consensus.AddValidator("validator2", 50000)
	require.NoError(t, err, "Should add validator2")
	
	// Select proposer
	proposer, err := consensus.SelectProposer()
	require.NoError(t, err, "Should select proposer")
	assert.NotEmpty(t, proposer, "Proposer should not be empty")
	assert.Contains(t, []string{"validator1", "validator2"}, proposer, 
		"Proposer should be one of the validators")
}

func TestABCIProtocol(t *testing.T) {
	err := Initialize()
	require.NoError(t, err)
	defer Shutdown()
	
	bc, err := NewBlockchain()
	require.NoError(t, err)
	defer bc.Destroy()
	
	t.Run("Info", func(t *testing.T) {
		request := types.ABCIRequest{
			Type: "Info",
		}
		
		response, err := bc.ProcessABCI(request)
		require.NoError(t, err)
		assert.Equal(t, "Info", response.Type)
		assert.Equal(t, uint64(0), response.Height)
	})
	
	t.Run("InitChain", func(t *testing.T) {
		request := types.ABCIRequest{
			Type: "InitChain",
			GenesisAccounts: []types.GenesisAccount{
				{Address: "alice", Balance: 1000000},
				{Address: "bob", Balance: 500000},
			},
		}
		
		response, err := bc.ProcessABCI(request)
		require.NoError(t, err)
		assert.Equal(t, "InitChainOk", response.Type)
		
		// Verify accounts were initialized
		balance, err := bc.GetBalance("alice")
		require.NoError(t, err)
		assert.Equal(t, uint64(1000000), balance)
	})
	
	t.Run("Query", func(t *testing.T) {
		request := types.ABCIRequest{
			Type: "Query",
			Path: "/height",
		}
		
		response, err := bc.ProcessABCI(request)
		require.NoError(t, err)
		assert.Equal(t, "Query", response.Type)
		assert.Equal(t, uint32(0), response.Code)
	})
}

func TestConcurrentAccess(t *testing.T) {
	err := Initialize()
	require.NoError(t, err)
	defer Shutdown()
	
	bc, err := NewBlockchain()
	require.NoError(t, err)
	defer bc.Destroy()
	
	err = bc.InitAccount("alice", 1000000)
	require.NoError(t, err)
	
	// Multiple goroutines reading balance concurrently
	done := make(chan bool, 10)
	for i := 0; i < 10; i++ {
		go func() {
			balance, err := bc.GetBalance("alice")
			assert.NoError(t, err)
			assert.Equal(t, uint64(1000000), balance)
			done <- true
		}()
	}
	
	// Wait for all goroutines
	for i := 0; i < 10; i++ {
		<-done
	}
}

func TestMultipleBlockchains(t *testing.T) {
	err := Initialize()
	require.NoError(t, err)
	defer Shutdown()
	
	// Create multiple blockchain instances
	bc1, err := NewBlockchain()
	require.NoError(t, err)
	defer bc1.Destroy()
	
	bc2, err := NewBlockchain()
	require.NoError(t, err)
	defer bc2.Destroy()
	
	// Initialize different accounts in each
	err = bc1.InitAccount("alice", 1000000)
	require.NoError(t, err)
	
	err = bc2.InitAccount("bob", 500000)
	require.NoError(t, err)
	
	// Verify isolation
	aliceBalance1, err := bc1.GetBalance("alice")
	require.NoError(t, err)
	assert.Equal(t, uint64(1000000), aliceBalance1)
	
	aliceBalance2, err := bc2.GetBalance("alice")
	require.NoError(t, err)
	assert.Equal(t, uint64(0), aliceBalance2, "alice shouldn't exist in bc2")
	
	bobBalance2, err := bc2.GetBalance("bob")
	require.NoError(t, err)
	assert.Equal(t, uint64(500000), bobBalance2)
}

func BenchmarkTransactionSubmission(b *testing.B) {
	Initialize()
	defer Shutdown()
	
	bc, _ := NewBlockchain()
	defer bc.Destroy()
	
	bc.InitAccount("alice", 10000000)
	bc.InitAccount("bob", 1000000)
	
	b.ResetTimer()
	
	for i := 0; i < b.N; i++ {
		tx := types.Transaction{
			From:      "alice",
			To:        "bob",
			Amount:    1000,
			GasFee:    0,
			Timestamp: uint64(time.Now().Unix()),
			Nonce:     uint64(i + 1),
		}
		bc.AddTransaction(tx)
	}
}

func BenchmarkBalanceQuery(b *testing.B) {
	Initialize()
	defer Shutdown()
	
	bc, _ := NewBlockchain()
	defer bc.Destroy()
	
	bc.InitAccount("alice", 1000000)
	
	b.ResetTimer()
	
	for i := 0; i < b.N; i++ {
		bc.GetBalance("alice")
	}
}
