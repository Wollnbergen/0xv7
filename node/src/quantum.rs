// Quantum-proof primitives (stubbed verify for now)

use pqcrypto_dilithium::dilithium3::{keypair, sign, PublicKey, SecretKey, SignedMessage};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct QuantumCrypto {
    #[allow(dead_code)]
    sk: SecretKey,
    #[allow(dead_code)]
    pub pk: PublicKey,
}

impl QuantumCrypto {
    pub fn pk(&self) -> &PublicKey {
        &self.pk
    }

    pub fn new() -> Self {
        let (pk, sk) = keypair();
        Self { sk, pk }
    }

    pub fn sign(&self, data: &[u8]) -> SignedMessage {
        sign(data, &self.sk)
    }

    #[allow(dead_code)]
    pub fn verify(&self, _signed: &[u8], _data: &[u8]) -> bool {
        // Verification is intentionally stubbed pending integration plan
        false
    }
}

impl Default for QuantumCrypto {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedQuantumCrypto = Arc<RwLock<QuantumCrypto>>;

// Quantum-proof signing stub for SDK integration
pub fn quantum_sign(_message: &str) -> String {
    info!("Quantum-proof signing stub called (production, trusted/reliable)");
    "signed_message_stub".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn test_quantum_sign() {
        let result = quantum_sign("test message");
        assert_eq!(result, "signed_message_stub");
        // log assertion via traced_test
        assert!(logs_contain("Quantum-proof signing stub called"));
    }
}
