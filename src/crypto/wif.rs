use crate::{crypto::hash::HashableForVec, neo_error::NeoError, types::Bytes, NeoRust};
use primitive_types::H256;
use sha2::{Digest, Sha256};
use std::hash::Hash;

pub trait Wif {
	fn to_wif(&self) -> String;
	// fn from_wif(&self, s: &str) -> Option<Vec<u8>>;
}

impl Wif for &[u8] {
	fn to_wif(&self) -> String {
		if self.len() != 32 {
			return String::new()
		}

		let mut extended = vec![0x80];
		extended.extend_from_slice(self);
		extended.push(0x01);

		let hash = Sha256::digest(&Sha256::digest(&extended));
		extended.extend_from_slice(&hash[0..4]);

		bs58::encode(extended.as_slice()).into_string()
	}
}

pub fn str_to_wif(s: &str) -> Result<Bytes, NeoError> {
	let data = bs58::encode(s).into_vec();

	if data.len() != 38 || data[0] != 0x80 || data[33] != 0x01 {
		return Err(NeoError::InvalidFormat)
	}

	let checksum = &data[..34].hash256()[..4];
	if checksum != &data[34..] {
		return Err(NeoError::InvalidPublicKey)
	}

	Ok(data[1..33].to_vec())
}
