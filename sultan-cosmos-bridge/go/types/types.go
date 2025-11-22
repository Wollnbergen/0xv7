package types

// Transaction represents a Sultan blockchain transaction
type Transaction struct {
	From      string `json:"from"`
	To        string `json:"to"`
	Amount    uint64 `json:"amount"`
	GasFee    uint64 `json:"gas_fee"`
	Timestamp uint64 `json:"timestamp"`
	Nonce     uint64 `json:"nonce"`
	Signature string `json:"signature,omitempty"`
}

// Block represents a Sultan blockchain block
type Block struct {
	Index        uint64        `json:"index"`
	Timestamp    uint64        `json:"timestamp"`
	Transactions []Transaction `json:"transactions"`
	PrevHash     string        `json:"prev_hash"`
	Hash         string        `json:"hash"`
	Nonce        uint64        `json:"nonce"`
	Validator    string        `json:"validator"`
	StateRoot    string        `json:"state_root"`
}

// Account represents an account in the Sultan blockchain
type Account struct {
	Address string `json:"address"`
	Balance uint64 `json:"balance"`
	Nonce   uint64 `json:"nonce"`
}

// Validator represents a validator in the consensus engine
type Validator struct {
	Address        string `json:"address"`
	Stake          uint64 `json:"stake"`
	VotingPower    uint64 `json:"voting_power"`
	IsActive       bool   `json:"is_active"`
	BlocksProposed uint64 `json:"blocks_proposed"`
	BlocksSigned   uint64 `json:"blocks_signed"`
}

// NodeStatus represents the current status of a Sultan node
type NodeStatus struct {
	Height         uint64 `json:"height"`
	LatestHash     string `json:"latest_hash"`
	ValidatorCount uint64 `json:"validator_count"`
	TotalAccounts  uint64 `json:"total_accounts"`
	PendingTxs     uint64 `json:"pending_txs"`
}

// ABCIRequest represents an ABCI protocol request
type ABCIRequest struct {
	Type            string              `json:"type"`
	Validators      []string            `json:"validators,omitempty"`
	GenesisAccounts []GenesisAccount    `json:"genesis_accounts,omitempty"`
	Height          uint64              `json:"height,omitempty"`
	Proposer        string              `json:"proposer,omitempty"`
	TxData          []byte              `json:"tx_data,omitempty"`
	Path            string              `json:"path,omitempty"`
	Data            []byte              `json:"data,omitempty"`
}

// ABCIResponse represents an ABCI protocol response
type ABCIResponse struct {
	Type             string   `json:"type"`
	Height           uint64   `json:"height,omitempty"`
	AppHash          string   `json:"app_hash,omitempty"`
	Code             uint32   `json:"code,omitempty"`
	Log              string   `json:"log,omitempty"`
	ValidatorUpdates []string `json:"validator_updates,omitempty"`
	Data             []byte   `json:"data,omitempty"`
	Value            []byte   `json:"value,omitempty"`
}

// GenesisAccount represents a genesis account configuration
type GenesisAccount struct {
	Address string `json:"address"`
	Balance uint64 `json:"balance"`
}

// BridgeError represents an error from the FFI bridge
type BridgeError struct {
	Code    int32  `json:"code"`
	Message string `json:"message"`
}

func (e *BridgeError) Error() string {
	return e.Message
}

// Error codes matching Rust side
const (
	ErrSuccess            = 0
	ErrNullPointer        = 1
	ErrInvalidUTF8        = 2
	ErrSerialization      = 3
	ErrDeserialization    = 4
	ErrBlockchain         = 5
	ErrConsensus          = 6
	ErrTransaction        = 7
	ErrState              = 8
	ErrInvalidParameter   = 9
	ErrInternal           = 10
)
