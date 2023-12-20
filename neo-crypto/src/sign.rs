use crate::{error::CryptoError, hash::HashableForVec};
use p256::{
	ecdsa::{
		signature::{Signer, Verifier},
		Signature, SigningKey, VerifyingKey,
	},
	SecretKey,
};

/// A struct that provides functions for signing and verifying messages using ECDSA with the P-256 curve.
pub struct Sign2 {}

impl Sign2 {
	/// Convert message in hexadecimal format to bytes and sign it
	pub fn sign_hex_message(
		message: &str,
		secret_key: &SecretKey,
	) -> Result<Signature, CryptoError> {
		let bytes = hex::decode(message).expect("Failed to decode hex");
		Self::sign_message(&bytes, secret_key)
	}

	/// Sign message in string format
	pub fn sign_message_string(
		message: &str,
		secret_key: &SecretKey,
	) -> Result<Signature, CryptoError> {
		let bytes = message.as_bytes().to_vec();
		Self::sign_message(&bytes, secret_key)
	}

	/// Sign message in byte format
	pub fn sign_message(message: &[u8], secret_key: &SecretKey) -> Result<Signature, CryptoError> {
		let signing_key = SigningKey::from(secret_key.clone());
		let hash = message.hash256();
		signing_key.try_sign(&hash).map_err(|_| CryptoError::SigningError)
	}

	/// Verify a given signature
	pub fn verify_signature(
		message: &[u8],
		signature: &Signature,
		verify_key: &VerifyingKey,
	) -> bool {
		let hash = message.hash256();
		verify_key.verify(&hash, signature).is_ok()
	}
}

/// A struct that represents a signature with its v, r, and s values.
pub struct SignatureData {
	v: u8,
	r: Vec<u8>,
	s: Vec<u8>,
}

impl SignatureData {
	/// Create a new SignatureData instance from a byte array with a given v value.
	pub fn from_bytes(v: u8, signature: &[u8]) -> Self {
		let r = signature[0..32].to_vec();
		let s = signature[32..64].to_vec();
		SignatureData { v, r, s }
	}

	/// Create a new SignatureData instance with given v, r, and s values.
	pub fn new(v: u8, r: &[u8], s: &[u8]) -> Self {
		SignatureData { v, r: r.to_vec(), s: s.to_vec() }
	}

	/// Get the concatenated r and s values.
	pub fn concatenated(&self) -> Vec<u8> {
		[self.r.clone(), self.s.clone()].concat()
	}

	/// Create a new SignatureData instance from a byte array with a given v value.
	pub fn from_byte_array_with_v(v: u8, signature: &[u8]) -> Self {
		let r = signature[0..32].to_vec();
		let s = signature[32..64].to_vec();
		SignatureData { v, r, s }
	}

	/// Create a new SignatureData instance from a byte array (v value is set to 0 by default).
	pub fn from_byte_array(signature: &[u8]) -> Self {
		Self::from_byte_array_with_v(0, signature)
	}
}

impl PartialEq for SignatureData {
	/// Implement PartialEq for equality checks.
	fn eq(&self, other: &Self) -> bool {
		self.v == other.v && self.r == other.r && self.s == other.s
	}
}

impl Eq for SignatureData {}
