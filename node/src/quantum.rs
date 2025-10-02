use pqcrypto_dilithium::dilithium3::{keypair, sign, PublicKey, SecretKey, SignedMessage};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct QuantumCrypto {
	#[allow(dead_code)] // For future quantum expansion
	sk: SecretKey, // Cached secret key for signing
	#[allow(dead_code)] // For future quantum expansion
	pub pk: PublicKey, // Cached public key for verify
}

impl QuantumCrypto {
	pub fn pk(&self) -> &PublicKey {
		&self.pk
	}
	pub fn new() -> Self {
		let (pk, sk) = keypair(); // Generate keypair
		Self { sk, pk }
	}

	pub fn sign(&self, data: &[u8]) -> SignedMessage {
		sign(data, &self.sk)
	}

	// Public getter for the quantum public key (for signature verification)

	#[allow(dead_code)] // For future quantum expansion
	pub fn verify(&self, _signed: &[u8], _data: &[u8]) -> bool {
		false
	}
}

pub type SharedQuantumCrypto = Arc<RwLock<QuantumCrypto>>;
