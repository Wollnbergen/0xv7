package slashing
import (
    "fmt"
)

import (
    "math/big"
    "time"
)

// SlashingModule handles validator penalties
type SlashingModule struct {
    DoubleSignSlashFraction    float64
    DowntimeSlashFraction      float64
    MinSignedBlocksWindow      int64
    MinSignedBlocksThreshold   float64
    DowntimeJailDuration       time.Duration
    SlashFractionDowntime      float64
}

// NewSlashingModule creates a new slashing module
func NewSlashingModule() *SlashingModule {
    return &SlashingModule{
        DoubleSignSlashFraction:  0.05,  // 5% slash for double signing
        DowntimeSlashFraction:    0.001, // 0.1% slash for downtime
        MinSignedBlocksWindow:    10000,
        MinSignedBlocksThreshold: 0.5,   // Must sign 50% of blocks
        DowntimeJailDuration:     10 * time.Minute,
        SlashFractionDowntime:    0.01,  // 1% slash
    }
}

// SlashingEvent represents a slashing event
type SlashingEvent struct {
    ValidatorAddress string
    SlashType       string
    Amount          *big.Int
    JailUntil       *time.Time
    Timestamp       time.Time
    Evidence        string
}

// HandleDoubleSign slashes a validator for double signing
func (sm *SlashingModule) HandleDoubleSign(validatorAddr string, stake *big.Int) *SlashingEvent {
    slashAmount := new(big.Int)
    slashAmount.Mul(stake, big.NewInt(int64(sm.DoubleSignSlashFraction*100)))
    slashAmount.Div(slashAmount, big.NewInt(100))
    
    jailTime := time.Now().Add(24 * time.Hour) // Jail for 24 hours
    
    return &SlashingEvent{
        ValidatorAddress: validatorAddr,
        SlashType:       "double_sign",
        Amount:          slashAmount,
        JailUntil:       &jailTime,
        Timestamp:       time.Now(),
        Evidence:        "Double signature detected",
    }
}

// HandleDowntime slashes a validator for being offline
func (sm *SlashingModule) HandleDowntime(validatorAddr string, stake *big.Int, missedBlocks int64) *SlashingEvent {
    slashAmount := new(big.Int)
    slashAmount.Mul(stake, big.NewInt(int64(sm.SlashFractionDowntime*100)))
    slashAmount.Div(slashAmount, big.NewInt(100))
    
    jailTime := time.Now().Add(sm.DowntimeJailDuration)
    
    return &SlashingEvent{
        ValidatorAddress: validatorAddr,
        SlashType:       "downtime",
        Amount:          slashAmount,
        JailUntil:       &jailTime,
        Timestamp:       time.Now(),
        Evidence:        fmt.Sprintf("Missed %d blocks", missedBlocks),
    }
}

// IsJailed checks if a validator is currently jailed
func (sm *SlashingModule) IsJailed(jailUntil *time.Time) bool {
    if jailUntil == nil {
        return false
    }
    return time.Now().Before(*jailUntil)
}
