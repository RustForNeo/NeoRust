use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use p256::ecdsa::signature::Verifier;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use crate::crypto::key_pair::KeyPair;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureData {
    pub v: u8,
    pub r: Vec<u8>,
    pub s: Vec<u8>,
}

impl SignatureData {

    pub fn new(v: u8, r: Vec<u8>, s: Vec<u8>) -> Self {
        Self { v, r, s }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let r = bytes[0..32].to_vec();
        let s = bytes[32..64].to_vec();
        Self { v: 0, r, s }
    }
}

pub fn sign_message(msg: &[u8], kp: &mut KeyPair) -> SignatureData {

    let sig = kp.sign(msg);
    let (r, s) = sig.split_scalars();
    SignatureData::from_bytes(&[r.to_bytes(), s.to_bytes()].concat())
}


// Get public key from private key
pub fn public_key(priv_key: &SigningKey) -> VerifyingKey {
    VerifyingKey::from(priv_key)
}

// Verify signature against public key
pub fn verify(msg: &[u8], sig: &SignatureData, pub_key: &VerifyingKey) -> bool {

    let sig = Signature::from_scalars(&sig.r, &sig.s).expect("valid sig");

    pub_key.verify(&msg, &sig).is_ok()
}