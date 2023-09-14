use p256::{ecdsa::{SigningKey, VerifyingKey}, U256};
use p256::ecdsa::Signature;
use p256::ecdsa::signature::{SignerMut, Verifier};
use p256::elliptic_curve::rand_core::OsRng;
use serde::{Deserialize, Serialize};

// Define a struct to represent the secp256r1 keypair
#[derive(Debug, PartialEq, Eq, Clone, Copy,Serialize, Deserialize)]
pub struct Secp256r1Keypair {
    private_key: SigningKey,
    public_key: VerifyingKey,
}

impl Secp256r1Keypair {
    // Constructor to generate a new keypair
    pub fn generate() -> Self {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = VerifyingKey::from(&private_key); //.to_encoded_point(false).as_bytes().to_owned();

        Self {
            private_key,
            public_key,
        }
    }

    // Sign a message with the private key
    pub fn sign(&mut self, message: &[u8]) -> Signature {
        self.private_key.sign(message)
    }

    // Verify a message signature with the public key
    pub fn verify(&self, message: &[u8], signature: &[u8; 64]) -> bool {
        self.public_key.verify(message, &Signature::from_slice(signature).unwrap()).is_ok()
    }
}
