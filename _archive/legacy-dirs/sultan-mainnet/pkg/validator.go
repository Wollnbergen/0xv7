package pkg

import (
    "crypto/ed25519"
    "encoding/hex"
    "sync"
)

type Validator struct {
    Address   string
    PublicKey ed25519.PublicKey
    Power     int64
    Active    bool
}

type ValidatorSet struct {
    validators map[string]*Validator
    mu         sync.RWMutex
}

func NewValidatorSet() *ValidatorSet {
    return &ValidatorSet{
        validators: make(map[string]*Validator),
    }
}

func (vs *ValidatorSet) AddValidator(address string, power int64) {
    vs.mu.Lock()
    defer vs.mu.Unlock()
    
    _, priv, _ := ed25519.GenerateKey(nil)
    pub := priv.Public().(ed25519.PublicKey)
    
    vs.validators[address] = &Validator{
        Address:   address,
        PublicKey: pub,
        Power:     power,
        Active:    true,
    }
}

func (vs *ValidatorSet) GetActiveValidators() []*Validator {
    vs.mu.RLock()
    defer vs.mu.RUnlock()
    
    var active []*Validator
    for _, v := range vs.validators {
        if v.Active {
            active = append(active, v)
        }
    }
    return active
}
