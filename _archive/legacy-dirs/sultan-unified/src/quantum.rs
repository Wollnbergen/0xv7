// Quantum-proof primitives (stubbed verify for now)

use pqcrypto_dilithium::dilithium3::{keypair, sign, open, PublicKey, SecretKey, SignedMessage};
use std::sync::Arc;
use tokio::sync::RwLock;
// tracing import removed; no stub logging in production

pub struct QuantumCrypto {
    #[allow(dead_code)]
    sk: SecretKey,
    #[allow(dead_code)]
    pub pk: PublicKey,
}

impl QuantumCrypto {
    pub fn pk(&self) -> &PublicKey { &self.pk }

    pub fn new() -> Self {
        let (pk, sk) = keypair();
        Self { sk, pk }
    }

    pub fn sign(&self, data: &[u8]) -> SignedMessage {
        sign(data, &self.sk)
    }

    pub fn verify(&self, signed: &SignedMessage) -> bool {
        open(signed, &self.pk).is_ok()
    }
}

impl Default for QuantumCrypto {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedQuantumCrypto = Arc<RwLock<QuantumCrypto>>;
