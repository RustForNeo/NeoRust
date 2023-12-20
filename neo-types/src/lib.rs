use base64::{engine::general_purpose, Engine};
use primitive_types::H256;
use serde_derive::{Deserialize, Serialize};
mod contract;
mod nns;

pub use contract::*;
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
pub use serde_with_utils::*;

pub mod error;
pub mod role;
pub mod script_hash;
pub mod stack_item;
pub mod string;
pub mod syncing;
pub mod txpool;
pub mod url_session;
pub mod util;
pub mod vm_state;
pub mod witness;

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
