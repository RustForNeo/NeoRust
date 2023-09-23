use crate::{
	crypto::{hash::HashableForVec, key_pair::KeyPair},
	neo_error::NeoError,
	types::{Bytes, PrivateKey},
};
use p256::{
	ecdsa::{
		signature::{digest::Mac, Signer, SignerMut},
		Signature,
	},
	PublicKey,
};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureData {
	pub v: u8,
	pub r: Bytes,
	pub s: Bytes,
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
	pub fn concatenated(&self) -> Bytes {
		let mut concatenated = Bytes::new();
		concatenated.extend_from_slice(&self.r);
		concatenated.extend_from_slice(&self.s);
		concatenated
	}

	pub fn sign_hex_message(
		hex_message: &str,
		key_pair: &mut KeyPair,
	) -> Result<SignatureData, NeoError> {
		let message = hex::decode(hex_message).unwrap();
		let sign = key_pair.private_key().sign(&message);
		Ok(SignatureData::from_bytes(sign))
	}

	pub fn sign_message(
		message: &Bytes,
		key_pair: &mut KeyPair,
	) -> Result<SignatureData, NeoError> {
		let signature = key_pair.private_key().sign(&message.hash256());

		let mut rec_id = None;
		for i in 0..4 {
			if let Some(key) =
				key_pair.public_key().recover(&i, &signature, &message.hash256()).unwrap()
			{
				if key == key_pair.public_key() {
					rec_id = Some(i);
					break
				}
			}
		}

		let rec_id = rec_id
			.ok_or(NeoError::Runtime("Could not construct recoverable key".to_string()))
			.unwrap();

		let v = 27 + rec_id;
		Ok(SignatureData::new(
			v as u8,
			signature.r.to_bytes_padded(32).unwrap(),
			signature.s.to_bytes_padded(32).unwrap(),
		))
	}
}

pub fn sign_message(msg: &[u8], kp: &mut KeyPair) -> SignatureData {
	let sig = kp.sign(msg).unwrap();
	let (r, s) = sig.split_scalars();
	SignatureData::from_bytes(&[r.to_bytes(), s.to_bytes()].concat())
}

// Get public key from private key
pub fn public_key(priv_key: &PrivateKey) -> PublicKey {
	PublicKey::from_secret_key(priv_key)
}

// Verify signature against public key
pub fn verify(msg: &[u8], sig: &SignatureData, pub_key: &PublicKey) -> bool {
	let sig = Signature::from_der(sig.concatenated().as_slice()).expect("valid sig");

	pub_key.verify(&msg, &sig).is_ok()
}
