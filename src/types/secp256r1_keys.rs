use core::fmt;
use serde::{self, Deserialize, Serialize};

#[cfg(feature = "openssl_support")]
use openssl::ec::EcKey;

#[cfg(feature = "ring_support")]
use ring::agreement;
#[cfg(feature = "ring_support")]
use ring::signature::{self, Signature};

#[cfg(feature = "substrate")]
use sp_std::prelude::*;

use core::mem::transmute;
use std::{convert::From, vec::Vec};

use serde_big_array::BigArray;

// big_array! { BigArray; }

#[derive(
	Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone,
)]
pub struct Secp256r1PublicKey {
	pub gx: [u8; 32],
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
	pub fn to_raw_bytes(&self) -> [u8; 64] {
		let mut bytes = [0_u8; 64];
		bytes[..32].copy_from_slice(&self.gx);
		bytes[32..].copy_from_slice(&self.gy);
		bytes
	}

	#[cfg(feature = "ring_support")]
	pub fn to_ring_agreement_key(&self) -> agreement::UnparsedPublicKey<Vec<u8>> {
		let buf = self.to_ring_bytes();
		agreement::UnparsedPublicKey::new(&agreement::ECDH_P256, buf.to_vec())
	}

	#[cfg(feature = "ring_support")]
	pub fn to_ring_signature_key(&self) -> signature::UnparsedPublicKey<Vec<u8>> {
		let buf = self.to_ring_bytes();
		signature::UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_FIXED, buf.to_vec())
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
}

impl Secp256r1PrivateKey {
	#[cfg(feature = "openssl_support")]
	pub fn from_der(der_bytes: &[u8]) -> Secp256r1PrivateKey {
		let eckey = EcKey::private_key_from_der(der_bytes).unwrap();
		let mut prvkey_bytes_le = eckey.private_key().to_vec();
		prvkey_bytes_le.reverse();
		let bytes_len = prvkey_bytes_le.len();
		// for private keys with leading 0s, 0s will not be reflected in the vec
		// pad it with 0s
		let num_pad_bytes = 32 - bytes_len;
		if num_pad_bytes > 0 {
			prvkey_bytes_le.resize(bytes_len + num_pad_bytes, 0);
		}
		let mut seed = [0_u8; 32];
		seed.copy_from_slice(prvkey_bytes_le.as_slice());

		Secp256r1PrivateKey { r: seed }
	}

	pub fn to_raw_bytes(&self) -> [u8; 32] {
		let mut bytes = [0_u8; 32];
		bytes[..32].copy_from_slice(&self.r);
		bytes
	}
}

impl Secp256r1Signature {
	#[cfg(feature = "ring_support")]
	pub fn from_ring_signature(ring_sig: &Signature) -> Secp256r1Signature {
		let ring_sig_buf = ring_sig.as_ref();
		assert_eq!(ring_sig_buf.len(), 64);

		let mut x: [u8; 32] = [0; 32];
		let mut y: [u8; 32] = [0; 32];
		x.copy_from_slice(&ring_sig_buf[..32]);
		y.copy_from_slice(&ring_sig_buf[32..]);
		x.reverse();
		y.reverse();
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
