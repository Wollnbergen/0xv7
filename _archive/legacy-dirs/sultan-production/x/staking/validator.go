package staking
import (
    "fmt"
)

import (
    "time"
    "math/big"
)

// Validator represents a network validator
type Validator struct {
    Address         string
    PublicKey       []byte
    StakedAmount    *big.Int
    Commission      float64 // 0-100%
    JailedUntil     *time.Time
    Delegators      map[string]*big.Int
    TotalDelegated  *big.Int
    Active          bool
}

// StakingModule manages validator staking
type StakingModule struct {
    Validators      map[string]*Validator
    MinStake        *big.Int
    MaxValidators   int
    UnbondingPeriod time.Duration
}

// NewStakingModule creates a new staking module
func NewStakingModule() *StakingModule {
    return &StakingModule{
        Validators:      make(map[string]*Validator),
        MinStake:        big.NewInt(32000000000), // 32 SLTN minimum
        MaxValidators:   100,
        UnbondingPeriod: 21 * 24 * time.Hour, // 21 days
    }
}

// CreateValidator registers a new validator
func (sm *StakingModule) CreateValidator(address string, pubKey []byte, stake *big.Int, commission float64) error {
    if stake.Cmp(sm.MinStake) < 0 {
        return fmt.Errorf("stake amount below minimum: %v < %v", stake, sm.MinStake)
    }
    
    validator := &Validator{
        Address:        address,
        PublicKey:      pubKey,
        StakedAmount:   stake,
        Commission:     commission,
        Delegators:     make(map[string]*big.Int),
        TotalDelegated: big.NewInt(0),
        Active:         true,
    }
    
    sm.Validators[address] = validator
    return nil
}

// Delegate allows users to delegate tokens to a validator
func (sm *StakingModule) Delegate(validatorAddr, delegatorAddr string, amount *big.Int) error {
    validator, exists := sm.Validators[validatorAddr]
    if !exists {
        return fmt.Errorf("validator not found: %s", validatorAddr)
    }
    
    if !validator.Active {
        return fmt.Errorf("validator is not active")
    }
    
    if current, ok := validator.Delegators[delegatorAddr]; ok {
        current.Add(current, amount)
    } else {
        validator.Delegators[delegatorAddr] = new(big.Int).Set(amount)
    }
    
    validator.TotalDelegated.Add(validator.TotalDelegated, amount)
    return nil
}

// GetValidatorSet returns the active validator set
func (sm *StakingModule) GetValidatorSet() []*Validator {
    var active []*Validator
    for _, v := range sm.Validators {
        if v.Active && v.JailedUntil == nil {
            active = append(active, v)
        }
    }
    return active
}
