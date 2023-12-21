use base64::{engine::general_purpose, Engine};
use elliptic_curve::sec1::ToEncodedPoint;
use ethereum_types::Address;
use p256::{ecdsa::VerifyingKey, AffinePoint};
use primitive_types::H256;
use serde_derive::{Deserialize, Serialize};
use std::{hash::Hash, ptr::hash};
mod contract;
mod nns;

pub use contract::*;
use neo_crypto::keys::{Secp256r1PrivateKey, Secp256r1PublicKey};
pub use nns::*;

pub mod address;
pub mod address_or_scripthash;
pub mod block;
pub mod bytes;
pub mod filter;
pub mod log;
pub mod numeric;
pub mod op_code;
pub mod path_or_string;
pub mod plugin_type;
pub mod serde_value;
pub mod serde_with_utils;
use crate::script_hash::ScriptHash;
pub use serde_with_utils::*;
pub mod error;
pub mod role;
pub mod script_hash;
pub mod stack_item;
pub mod string;
pub mod syncing;
pub mod tx_pool;
pub mod url_session;
pub mod util;
pub mod vm_state;

pub type Byte = u8;
pub type Bytes = Vec<u8>;
pub type TxHash = H256;

pub trait ExternBase64 {
	fn to_base64(&self) -> String;
}

impl ExternBase64 for String {
	fn to_base64(&self) -> String {
		general_purpose::STANDARD.encode(self.as_bytes())
	}
}

// ScryptParams
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScryptParamsDef {
	pub log_n: u8,
	pub r: u32,
	pub p: u32,
}

impl Default for ScryptParamsDef {
	fn default() -> Self {
		Self { log_n: 14, r: 8, p: 8 }
	}
}

// Extend Vec<u8> with a to_base64 method
pub trait Base64Encode {
	fn to_base64(&self) -> String;
}

impl Base64Encode for Vec<u8> {
	fn to_base64(&self) -> String {
		base64::encode(&self)
	}
}

impl Base64Encode for &[u8] {
	fn to_base64(&self) -> String {
		base64::encode(&self)
	}
}

pub fn secret_key_to_script_hash(secret_key: &Secp256r1PrivateKey) -> ScriptHash {
	let public_key = secret_key.to_public_key().unwrap();
	public_key_to_script_hash(&public_key)
}

pub fn public_key_to_script_hash(pubkey: &Secp256r1PublicKey) -> ScriptHash {
	raw_public_key_to_script_hash(&pubkey.to_raw_bytes()[1..])
}

pub fn raw_public_key_to_script_hash<T: AsRef<[u8]>>(pubkey: T) -> ScriptHash {
	let pubkey = pubkey.as_ref();
	assert_eq!(pubkey.len(), 64, "raw public key must be 64 bytes");
	let digest = vec![]; // keccak256(pubkey);
	ScriptHash::from_slice(&digest)
}

pub fn to_checksum(addr: &Address, chain_id: Option<u8>) -> String {
	let prefixed_addr = match chain_id {
		Some(chain_id) => format!("{chain_id}0x{addr:x}"),
		None => format!("{addr:x}"),
	};
	let hash = hex::encode(prefixed_addr);
	let hash = hash.as_bytes();

	let addr_hex = hex::encode(addr.as_bytes());
	let addr_hex = addr_hex.as_bytes();

	addr_hex.iter().zip(hash).fold("0x".to_owned(), |mut encoded, (addr, hash)| {
		encoded.push(if *hash >= 56 {
			addr.to_ascii_uppercase() as char
		} else {
			addr.to_ascii_lowercase() as char
		});
		encoded
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use hex;
	use rustc_serialize::base64::FromBase64;

	#[test]
	fn test_base64_encode_bytes() {
		let input = hex::decode("150c14242dbf5e2f6ac2568b59b7822278d571b75f17be0c14242dbf5e2f6ac2568b59b7822278d571b75f17be13c00c087472616e736665720c14897720d8cd76f4f00abfa37c0edd889c208fde9b41627d5b5238").unwrap();
		let expected = "FQwUJC2/Xi9qwlaLWbeCInjVcbdfF74MFCQtv14vasJWi1m3giJ41XG3Xxe+E8AMCHRyYW5zZmVyDBSJdyDYzXb08Aq/o3wO3YicII/em0FifVtSOA==";

		let encoded = input.to_base64();

		assert_eq!(encoded, expected);
	}

	#[test]
	fn test_base64_decode() {
		let encoded = "FQwUJC2/Xi9qwlaLWbeCInjVcbdfF74MFCQtv14vasJWi1m3giJ41XG3Xxe+E8AMCHRyYW5zZmVyDBSJdyDYzXb08Aq/o3wO3YicII/em0FifVtSOA==";
		let expected = "150c14242dbf5e2f6ac2568b59b7822278d571b75f17be0c14242dbf5e2f6ac2568b59b7822278d571b75f17be13c00c087472616e736665720c14897720d8cd76f4f00abfa37c0edd889c208fde9b41627d5b5238";

		let decoded = encoded.from_base64().unwrap();
		let decoded_hex = hex::encode(decoded);

		assert_eq!(decoded_hex, expected);
	}
}
