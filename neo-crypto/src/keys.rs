use crate::error::CryptoError;
use bytes::Bytes;
use core::fmt;
use p256::{
	ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey},
	elliptic_curve::{generic_array::GenericArray, sec1::ToEncodedPoint, Field},
};
use primitive_types::U256;
use rand_core::OsRng;
use rlp::DecoderError;
use serde::{Deserialize, Serialize};
use signature::{Keypair, SignerMut, Verifier};
use std::collections::BTreeMap;
use typenum::U32;

#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(
	Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone,
)]
pub struct Secp256r1PublicKey {
	// #[serde(with = "BigArray")]
	pub gx: [u8; 32],
	// #[serde(with = "BigArray")]
	pub gy: [u8; 32],
}

#[derive(
	Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone,
)]
pub struct Secp256r1PrivateKey {
	pub r: [u8; 32],
}

#[derive(
	Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone,
)]
pub struct Secp256r1Signature {
	pub x: [u8; 32],
	pub y: [u8; 32],
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
pub struct Secp256r1SignedMsg<T: Serialize> {
	pub msg: T,
	pub signature: Secp256r1Signature,
}

impl Secp256r1PublicKey {
	// Creates a public key from a byte slice
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
		if bytes.len() != 65 || bytes[0] != 0x04 {
			return Err(CryptoError::InvalidPublicKey)
		}

		let mut gx = [0u8; 32];
		let mut gy = [0u8; 32];
		gx.copy_from_slice(&bytes[1..33]);
		gy.copy_from_slice(&bytes[33..65]);

		Ok(Secp256r1PublicKey { gx, gy })
	}
	pub fn to_raw_bytes(&self) -> [u8; 64] {
		let mut bytes = [0_u8; 64];
		bytes[..32].copy_from_slice(&self.gx);
		bytes[32..].copy_from_slice(&self.gy);
		bytes
	}

	pub fn to_ring_bytes(&self) -> [u8; 65] {
		let mut buf = [0_u8; 65];
		buf[0] = 4;
		buf[1..33].copy_from_slice(&self.gx);
		buf[1..33].reverse();
		buf[33..].copy_from_slice(&self.gy);
		buf[33..].reverse();
		buf
	}

	// Verifies a signature against a message
	pub fn verify(
		&self,
		message: &[u8],
		signature: &Secp256r1Signature,
	) -> Result<(), CryptoError> {
		let gx_gy_bytes = [self.gx, self.gy].concat();
		let verifying_key = VerifyingKey::from_sec1_bytes(&gx_gy_bytes)
			.map_err(|_| CryptoError::InvalidPublicKey)?;

		let signature = Signature::from_scalars(signature.x, signature.y)
			.map_err(|_| CryptoError::SigningError)?;

		verifying_key
			.verify(message, &signature)
			.map_err(|_| CryptoError::SignatureVerificationError)
	}
}

impl Secp256r1PrivateKey {
	// Generates a new private key using the provided RNG
	pub fn random(rng: &mut OsRng) -> Self {
		let signing_key = Secp256r1PrivateKey::random(rng);
		let scalar_bytes = signing_key.to_raw_bytes();

		let mut r = [0u8; 32];
		r.copy_from_slice(&scalar_bytes);

		Secp256r1PrivateKey { r }
	}
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
		if bytes.len() != 32 {
			return Err(CryptoError::InvalidPrivateKey)
		}

		let mut arr = [0u8; 32];
		arr.copy_from_slice(bytes);
		Ok(Self { r: arr })
	}

	pub fn to_raw_bytes(&self) -> [u8; 32] {
		let mut bytes = [0_u8; 32];
		bytes[..32].copy_from_slice(&self.r);
		bytes
	}

	// Converts a private key to a public key
	pub fn to_public_key(&self) -> Result<Secp256r1PublicKey, CryptoError> {
		let private_key_bytes = GenericArray::<u8, U32>::from_slice(&self.r);
		let signing_key = SigningKey::from_bytes(private_key_bytes)
			.map_err(|_| CryptoError::InvalidPrivateKey)?;

		let verifying_key = signing_key.verifying_key();
		let encoded_point = verifying_key.to_encoded_point(false);

		if encoded_point.is_compressed() || encoded_point.len() != 65 {
			return Err(CryptoError::InvalidPublicKey)
		}

		let bytes = encoded_point.as_bytes();
		let mut gx = [0u8; 32];
		let mut gy = [0u8; 32];
		gx.copy_from_slice(&bytes[1..33]);
		gy.copy_from_slice(&bytes[33..65]);

		Ok(Secp256r1PublicKey { gx, gy })
	}

	pub fn sign_tx(&self, message: &[u8]) -> Result<Secp256r1Signature, CryptoError> {
		let private_key_bytes = GenericArray::<u8, U32>::from_slice(&self.r);
		let signing_key = SigningKey::from_bytes(private_key_bytes)
			.map_err(|_| CryptoError::InvalidPrivateKey)?;

		let (signature, _) =
			signing_key.try_sign(message).map_err(|_| CryptoError::SigningError)?;

		let signature_bytes = signature.to_bytes();

		let mut x = [0u8; 32];
		let mut y = [0u8; 32];
		x.copy_from_slice(&signature_bytes[..32]);
		y.copy_from_slice(&signature_bytes[32..]);

		Ok(Secp256r1Signature { x, y })
	}
}

impl Secp256r1Signature {
	pub fn from_u256(r: U256, s: U256) -> Self {
		let mut x = [0u8; 32];
		let mut y = [0u8; 32];
		r.to_big_endian(&mut x);
		s.to_big_endian(&mut y);
		Secp256r1Signature { x, y }
	}
	pub fn to_ring_signature_bytes(&self) -> [u8; 64] {
		let mut temp_buf: [u8; 64] = [0; 64];
		temp_buf[..32].copy_from_slice(&self.x);
		temp_buf[32..].copy_from_slice(&self.y);
		temp_buf[..32].reverse();
		temp_buf[32..].reverse();
		temp_buf
	}

	pub fn to_raw_bytes(&self) -> [u8; 64] {
		let mut bytes = [0_u8; 64];
		bytes[..32].copy_from_slice(&self.x);
		bytes[32..].copy_from_slice(&self.y);
		bytes
	}
}

impl fmt::Display for Secp256r1PrivateKey {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Secp256r1PrivateKey\n").unwrap();
		write!(f, "r: {}\n", hex::encode(self.r))
	}
}

impl fmt::Display for Secp256r1PublicKey {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Secp256r1PublicKey\n").unwrap();
		write!(f, "gx: {}\n", hex::encode(self.gx)).unwrap();
		write!(f, "gy: {}\n", hex::encode(self.gy))
	}
}

impl fmt::Display for Secp256r1Signature {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Secp256r1Signature\n").unwrap();
		write!(f, "x: {}\n", hex::encode(self.x)).unwrap();
		write!(f, "y: {}\n", hex::encode(self.y))
	}
}

pub trait PrivateKeyExtension
where
	Self: Sized,
{
	fn to_vec(&self) -> Vec<u8>;

	fn from_slice(slice: &[u8]) -> Result<Self, CryptoError>;
}

impl PrivateKeyExtension for Secp256r1PrivateKey {
	fn to_vec(&self) -> Vec<u8> {
		self.to_raw_bytes().to_vec()
	}

	fn from_slice(slice: &[u8]) -> Result<Self, CryptoError> {
		if slice.len() != 32 {
			return Err(CryptoError::InvalidPublicKey)
		}

		let mut arr = [0u8; 32];
		arr.copy_from_slice(slice);
		Ok(Self::from_bytes(&arr).map_err(|_| CryptoError::InvalidPublicKey).unwrap())
	}
}

pub trait PublicKeyExtension
where
	Self: Sized,
{
	fn to_vec(&self) -> Vec<u8>;
	fn from_slice(slice: &[u8]) -> Result<Self, CryptoError>;
}

impl PublicKeyExtension for Secp256r1PublicKey {
	fn to_vec(&self) -> Vec<u8> {
		self.to_raw_bytes().to_vec()
	}
	fn from_slice(slice: &[u8]) -> Result<Self, CryptoError> {
		if slice.len() != 65 && slice.len() != 33 {
			return Err(CryptoError::InvalidPublicKey)
		}
		Self::from_slice(slice).map_err(|_| CryptoError::InvalidPublicKey)
	}
}
