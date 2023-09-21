use p256::{
	ecdsa::{
		signature::{SignerMut, Verifier},
		Signature, VerifyingKey,
	},
	elliptic_curve::rand_core::OsRng,
};

use crate::types::PrivateKey;
use getset::{CopyGetters, Getters, MutGetters, Setters};

// Define a struct to represent the secp256r1 keypair
#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters)]
pub struct Secp256r1Keypair {
	#[getset(get = "pub", set = "pub", get_mut = "pub")]
	private_key: PrivateKey,
	#[getset(get_copy = "pub", set = "pub", get_mut = "pub")]
	public_key: VerifyingKey,
}

impl Secp256r1Keypair {
	// Constructor to generate a new keypair
	pub fn generate() -> Self {
		let private_key = PrivateKey::random(&mut OsRng);
		let public_key = VerifyingKey::from(&private_key); //.to_encoded_point(false).as_bytes().to_owned();

		Self { private_key, public_key }
	}

	// Sign a message with the private key
	pub fn sign(&mut self, message: &[u8]) -> Signature {
		self.private_key.sign(message)
	}

	// Verify a message signature with the public key
	pub fn verify(&self, message: &[u8], signature: &[u8; 64]) -> bool {
		self.public_key
			.verify(message, &Signature::from_der(signature).unwrap())
			.is_ok()
	}
}
