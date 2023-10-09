//! # KeyPair
//!
//! `KeyPair` is a module that provides an implementation for Elliptic Curve Key Pairs using the `p256` crate.
//!
//! This structure can be used to manage and manipulate EC key pairs,
//! including generating new pairs, importing them from raw bytes,
//! and converting them to various formats.

use crate::error::CryptoError;
use p256::{
	elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint},
	EncodedPoint, NonZeroScalar, PublicKey, SecretKey,
};
use primitive_types::H160;
use rand::rngs::OsRng;

/// Represents an Elliptic Curve Key Pair containing both a private and a public key.
pub struct KeyPair {
	/// The private key component of the key pair.
	pub private_key: SecretKey,

	/// The public key component of the key pair.
	pub public_key: PublicKey,
}

impl KeyPair {
	/// Creates a new `KeyPair` instance given a private key and its corresponding public key.
	///
	/// # Arguments
	///
	/// * `private_key` - A `SecretKey` representing the private key.
	/// * `public_key` - A `PublicKey` representing the public key.
	pub fn new(private_key: SecretKey, public_key: PublicKey) -> Self {
		Self { private_key, public_key }
	}

	/// Derives a new `KeyPair` instance from just a private key.
	/// The public key is derived from the given private key.
	///
	/// # Arguments
	///
	/// * `private_key` - A `SecretKey` representing the private key.
	pub fn from_secret_key(private_key: SecretKey) -> Self {
		let scalar = NonZeroScalar::from(&private_key);
		let public_key = PublicKey::from_secret_scalar(&scalar);
		Self::new(private_key, public_key)
	}

	/// Returns the 32-byte representation of the private key.
	pub fn private_key_bytes(&self) -> [u8; 32] {
		self.private_key.to_bytes().into()
	}

	/// Returns the 65-byte uncompressed representation of the public key.
	pub fn public_key_bytes(&self) -> [u8; 65] {
		let mut buf = [0u8; 65];
		// Convert the PublicKey to its byte representation
		let vec_bytes: Vec<u8> = self.public_key.to_encoded_point(false).as_bytes().to_vec(); // uncompressed form
		buf.copy_from_slice(&vec_bytes[0..65]);

		buf
	}
}

impl KeyPair {
	/// Generates a new random `KeyPair`.
	pub fn new_random() -> Self {
		let mut rng = OsRng; // A cryptographically secure random number generator
		let secret_key = p256::SecretKey::random(&mut rng);
		Self::from_secret_key(secret_key)
	}

	/// Creates an `KeyPair` from a given 32-byte private key.
	///
	/// # Arguments
	///
	/// * `private_key` - A 32-byte slice representing the private key.
	pub fn from_private_key(private_key: &[u8; 32]) -> Result<Self, CryptoError> {
		let secret_key = SecretKey::from_bytes(private_key.into())?;
		Ok(Self::from_secret_key(secret_key))
	}

	/// Creates an `KeyPair` from a given 65-byte public key.
	/// This will use a dummy private key internally.
	///
	/// # Arguments
	///
	/// * `public_key` - A 65-byte slice representing the uncompressed public key.
	pub fn from_public_key(public_key: &[u8; 65]) -> Result<Self, CryptoError> {
		let public_key = PublicKey::from_sec1_bytes(public_key)?;
		let secret_key = SecretKey::from_bytes((&[0u8; 32]).into()).unwrap(); // dummy private key
		Ok(Self::new(secret_key, public_key))
	}
}
