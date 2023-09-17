// String.rs

use base64::{decode, encode};
use hex::FromHex;
use primitive_types::H160;
trait NeoString {
	fn to_vec(&self) -> Vec<u8>;
	fn to_hex(&self) -> String;
	fn base64_decode(&self) -> Vec<u8>;
	fn base64_encode(&self) -> String;
	fn var_size(&self) -> usize;
	fn is_valid_hex(&self) -> bool;
}

impl NeoString for String {
	fn to_vec(&self) -> Vec<u8> {
		Vec::from_hex(self).unwrap()
	}

	fn to_hex(&self) -> String {
		hex::encode(self)
	}

	fn base64_decode(&self) -> Vec<u8> {
		decode(self).unwrap()
	}

	fn base64_encode(&self) -> String {
		encode(self)
	}

	fn var_size(&self) -> usize {
		self.len()
	}

	fn is_valid_hex(&self) -> bool {
		self.from_hex().is_ok()
	}
}
