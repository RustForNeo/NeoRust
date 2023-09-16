use std::hash::Hash;
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use p256::ecdsa::signature::{Signer, Verifier};
use p256::elliptic_curve::rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use crate::types::Address;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyPair {
    pub private_key: SigningKey,
    pub public_key: VerifyingKey,
}

impl KeyPair {

    pub fn new(private_key: SigningKey, public_key: VerifyingKey) -> Self {
        Self { private_key, public_key }
    }

    pub fn from_private_key(private_key: &SigningKey) -> Self {
        let public_key = VerifyingKey::from(private_key);
        Self::new(private_key.clone(), public_key)
    }

    pub fn generate() -> Self {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = VerifyingKey::from(&private_key); //.to_encoded_point(false).as_bytes().to_owned();

        Self {
            private_key,
            public_key,
        }
    }

    // Sign a message with the private key
    pub fn sign(&mut self, message: &[u8]) -> (Signature, RecoveryId) {
        self.private_key.sign(message)
    }

    // Verify a message signature with the public key
    pub fn verify(&self, message: &[u8], signature: &[u8; 64]) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(message);
        let hash = hasher.finalize().as_slice();
        self.public_key.verify(hash, &Signature::from_slice(signature).unwrap()).is_ok()
    }

    pub fn get_address(&self) -> Address {
        Address::from_public_key(&self.public_key)
    }

    fn private_key(&self) -> SigningKey {
        self.private_key.clone()
    }

    fn public_key(&self) -> VerifyingKey {
        self.public_key.clone()
    }

}