package crypto

import (
    "crypto/ed25519"
    "crypto/rand"
    "encoding/hex"
    "fmt"
)

// KeyPair represents an ed25519 key pair
type KeyPair struct {
    PublicKey  ed25519.PublicKey
    PrivateKey ed25519.PrivateKey
}

// GenerateKeyPair creates a new ed25519 key pair
func GenerateKeyPair() (*KeyPair, error) {
    pub, priv, err := ed25519.GenerateKey(rand.Reader)
    if err != nil {
        return nil, fmt.Errorf("failed to generate key pair: %w", err)
    }
    
    return &KeyPair{
        PublicKey:  pub,
        PrivateKey: priv,
    }, nil
}

// Sign signs a message with the private key
func (kp *KeyPair) Sign(message []byte) []byte {
    return ed25519.Sign(kp.PrivateKey, message)
}

// Verify verifies a signature with a public key
func Verify(publicKey ed25519.PublicKey, message, signature []byte) bool {
    return ed25519.Verify(publicKey, message, signature)
}

// PublicKeyToAddress converts public key to address
func PublicKeyToAddress(pubKey ed25519.PublicKey) string {
    return "sultan1" + hex.EncodeToString(pubKey[:20])
}
