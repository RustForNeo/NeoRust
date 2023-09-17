use crate::protocol::neo_config::DEFAULT_ADDRESS_VERSION;
use bitcoin::base58;
use num_bigint::{BigInt, Sign};
use p256::pkcs8::der::Encode;
use ripemd::Digest;
use sha2::Sha256;
use std::convert::TryInto;

pub trait BytesExtern {
	fn to_hex(&self) -> String;

	fn base58check_encode(&self) -> String;

	fn scripthash_to_address(&self) -> String;

	fn to_padded(&self, size: usize, trailing: bool) -> Vec<u8>;

	fn trim_trailing(&self, byte: u8) -> &[u8];

	fn to_num<T: TryFrom<[u8]>>(&self) -> T;

	fn xor(self, other: Vec<u8>) -> Vec<u8>;

	fn hash256(&self) -> Vec<u8>;

	fn ripemd160(&self) -> Vec<u8>;

	fn sha256_ripemd160(&self) -> Vec<u8>;

	fn to_bint(&self) -> BigInt;

	fn base64_encode(&self) -> String;

	fn is_between(byte: u8, min: u8, max: u8) -> bool;
}

impl BytesExtern for [u8] {
	fn to_hex(&self) -> String {
		hex::encode(self)
	}

	fn base58check_encode(&self) -> String {
		base58::encode_check(self)
	}

	fn scripthash_to_address(&self) -> String {}

	fn to_padded(&self, size: usize, trailing: bool) -> Vec<u8> {
		let len = self.len();
		if &len > &size {
			return vec![]
		}
		let mut padded = vec![0; size - len];
		if trailing {
			padded.extend_from_slice(self);
		} else {
			padded.extend_from_slice(&self[..len]);
		}
		padded
	}

	fn trim_trailing(&self, byte: u8) -> &[u8] {
		let mut slice = self;
		while let Some(&last) = slice.last() {
			if &last != &byte {
				break
			}
			slice = &slice[..slice.len() - 1].to_vec();
		}
		slice
	}

	fn to_num<T: TryFrom<[u8]>>(&self) -> T {
		let bytes = self.try_into().unwrap();
		T::from_le_bytes(bytes)
	}

	fn xor(self, other: Vec<u8>) -> Vec<u8> {
		assert_eq!(self.len(), other.len());
		self.iter().zip(other.iter()).map(|(a, b)| a ^ b).collect()
	}

	fn hash256(&self) -> Vec<u8> {
		let mut hasher = Sha256::new();
		hasher.update(self);
		hasher.finalize().to_vec().unwrap()
	}

	fn ripemd160(&self) -> Vec<u8> {
		let mut hasher = ripemd::Digest::new();
		hasher.update(self);
		hasher.finalize().to_vec()
	}

	fn sha256_ripemd160(&self) -> Vec<u8> {
		let sha256 = self.hash256();
		sha256.ripemd160()
	}

	fn to_bint(&self) -> BigInt {
		BigInt::from_bytes_le(Sign::NoSign, self)
	}

	fn base64_encode(&self) -> String {
		base64::encode(self)
	}

	fn is_between(byte: u8, min: u8, max: u8) -> bool {
		&byte >= &min && byte <= max
	}
}

impl BytesExtern for Vec<u8> {
	fn to_hex(&self) -> String {
		hex::encode(self)
	}

	fn base58check_encode(&self) -> String {
		base58::encode(self).into_string()
	}

	fn scripthash_to_address(&self) -> String {
		let script = [DEFAULT_ADDRESS_VERSION].iter().chain(self.iter().rev()).collect();
		let checksum = hash256(&script)[..4].to_vec();
		base58::encode(script.iter().chain(checksum.iter()).copied()).into_string()
	}

	fn to_padded(&self, size: usize, trailing: bool) -> Vec<u8> {
		let len = self.len();
		if &len > &size {
			return vec![]
		}
		let mut padded = vec![0; size - len];
		if trailing {
			padded.extend_from_slice(self);
		} else {
			padded.extend_from_slice(&self[..len]);
		}
		padded
	}

	fn trim_trailing(&self, byte: u8) -> &[u8] {
		let mut slice = self;
		while let Some(&last) = slice.last() {
			if &last != &byte {
				break
			}
			slice = &slice[..slice.len() - 1].to_vec();
		}
		slice
	}

	fn to_num<T: TryFrom<[u8]>>(&self) -> T {
		let bytes = self.try_into().unwrap();
		T::from_le_bytes(bytes)
	}

	fn xor(self, other: Vec<u8>) -> Vec<u8> {
		assert_eq!(self.len(), other.len());
		self.iter().zip(other.iter()).map(|(a, b)| a ^ b).collect()
	}

	fn hash256(&self) -> Vec<u8> {
		let mut hasher = Sha256::new();
		hasher.update(self);
		hasher.finalize().to_vec().unwrap()
	}

	fn ripemd160(&self) -> Vec<u8> {
		let mut hasher = ripemd::Digest::new();
		hasher.update(self);
		hasher.finalize().to_vec()
	}

	fn sha256_ripemd160(&self) -> Vec<u8> {
		let sha256 = self.hash256();
		sha256.ripemd160()
	}

	fn to_bint(&self) -> BigInt {
		BigInt::from_bytes_le(Sign::NoSign, self)
	}

	fn base64_encode(&self) -> String {
		base64::encode(self)
	}

	fn is_between(byte: u8, min: u8, max: u8) -> bool {
		&byte >= &min && byte <= max
	}
}

// Other implementations
