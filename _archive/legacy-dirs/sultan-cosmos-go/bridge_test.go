package sultancosmos

import (
	"testing"
)

// TestBridgeInitialization verifies Sultan bridge can be created and destroyed
func TestBridgeInitialization(t *testing.T) {
	bridge, err := NewSultanBridge()
	if err != nil {
		t.Fatalf("Failed to create bridge: %v", err)
	}
	defer bridge.Close()
	
	if bridge.blockchainHandle == 0 {
		t.Fatal("Blockchain handle is 0 after initialization")
	}
	
	if bridge.consensusHandle == 0 {
		t.Fatal("Consensus handle is 0 after initialization")
	}
}

// TestGetHeight tests blockchain height query
func TestGetHeight(t *testing.T) {
	bridge, err := NewSultanBridge()
	if err != nil {
		t.Fatalf("Failed to create bridge: %v", err)
	}
	defer bridge.Close()
	
	height, err := bridge.GetHeight()
	if err != nil {
		t.Fatalf("Failed to get height: %v", err)
	}
	
	t.Logf("Current blockchain height: %d", height)
}

// TestGetBalance tests balance query
func TestGetBalance(t *testing.T) {
	bridge, err := NewSultanBridge()
	if err != nil {
		t.Fatalf("Failed to create bridge: %v", err)
	}
	defer bridge.Close()
	
	// Initialize a test account
	err = bridge.InitAccount("test_account", 1000000) // 1 SLTN
	if err != nil {
		t.Fatalf("Failed to init account: %v", err)
	}
	
	// Query balance
	balance, err := bridge.GetBalance("test_account")
	if err != nil {
		t.Fatalf("Failed to get balance: %v", err)
	}
	
	if balance != 1000000 {
		t.Errorf("Expected balance 1000000, got %d", balance)
	}
}

// TestGetLatestHash tests latest block hash query
func TestGetLatestHash(t *testing.T) {
	bridge, err := NewSultanBridge()
	if err != nil {
		t.Fatalf("Failed to create bridge: %v", err)
	}
	defer bridge.Close()
	
	hash, err := bridge.GetLatestHash()
	if err != nil {
		t.Fatalf("Failed to get latest hash: %v", err)
	}
	
	t.Logf("Latest block hash: %s", hash)
}

// TestAddValidator tests adding a new validator
func TestAddValidator(t *testing.T) {
	bridge, err := NewSultanBridge()
	if err != nil {
		t.Fatalf("Failed to create bridge: %v", err)
	}
	defer bridge.Close()
	
	// Add validator with 10,000 SLTN stake (minimum)
	err = bridge.AddValidator("validator1", 10000000000) // 10k SLTN in usltn
	if err != nil {
		t.Fatalf("Failed to add validator: %v", err)
	}
	
	t.Log("Validator added successfully")
}

// Benchmark bridge initialization overhead
func BenchmarkBridgeInit(b *testing.B) {
	for i := 0; i < b.N; i++ {
		bridge, err := NewSultanBridge()
		if err != nil {
			b.Fatalf("Failed to create bridge: %v", err)
		}
		bridge.Close()
	}
}

// Benchmark balance queries
func BenchmarkGetBalance(b *testing.B) {
	bridge, err := NewSultanBridge()
	if err != nil {
		b.Fatalf("Failed to create bridge: %v", err)
	}
	defer bridge.Close()
	
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := bridge.GetBalance("genesis")
		if err != nil {
			b.Fatalf("Failed to get balance: %v", err)
		}
	}
}
