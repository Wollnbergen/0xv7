use pqcrypto_dilithium::dilithium3::{keypair, sign, PublicKey, SecretKey, SignedMessage};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct QuantumCrypto {
	sk: SecretKey, // Cached secret key for signing
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

	// pub fn verify(&self, _signed: &[u8], _data: &[u8]) -> bool { // Unused for now
	//     // PHASE 3: Quantum verification disabled for build. Refactor to store SignedMessage in Transaction for production.
	//     // let signed_msg = SignedMessage(signed.to_vec());
	//     true // Always pass for build
	// }
}

pub type SharedQuantumCrypto = Arc<RwLock<QuantumCrypto>>;
