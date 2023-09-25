use crate::{
	crypto::hash::HashableForVec, neo_error::NeoError,
	protocol::neo_config::DEFAULT_ADDRESS_VERSION, script::script_builder::ScriptBuilder,
	types::PublicKey,
};
use hex::FromHexError;
use primitive_types::H160;

pub trait ScriptHashExtension
where
	Self: Sized,
{
	fn to_string(&self) -> String;

	fn from_slice(slice: &[u8]) -> Result<Self, NeoError>;

	fn from_hex(hex: &str) -> Result<Self, hex::FromHexError>;
	fn from_address(address: &str) -> Result<Self, NeoError>;

	fn from_public_key(public_key: &PublicKey) -> Self;

	fn from_public_keys(public_keys: &mut [PublicKey], threshold: usize) -> Self {
		let mut script =
			ScriptBuilder::build_multisig_script(public_keys, threshold as u8).unwrap();
		Self::from_script(&script)
	}

	fn to_address(&self) -> String;
	fn to_vec(&self) -> Vec<u8>;

	fn to_le_vec(&self) -> Vec<u8> {
		let mut vec = self.to_vec();
		vec.reverse();
		vec
	}

	fn from_script(script: &[u8]) -> Self;
}

impl ScriptHashExtension for H160 {
	fn to_string(&self) -> String {
		bs58::encode(self.0).into_string()
	}

	fn from_slice(slice: &[u8]) -> Result<Self, NeoError> {
		if slice.len() != 20 {
			return Err(NeoError::InvalidAddress)
		}

		let mut arr = [0u8; 20];
		arr.copy_from_slice(slice);
		Ok(Self(arr))
	}

	fn from_hex(hex: &str) -> Result<Self, FromHexError> {
		// remove the '0x' prefix if it exists
		let hex = if hex.starts_with("0x") { &hex[2..] } else { hex };

		let bytes = hex::decode(hex)?;
		Ok(Self::from_slice(&bytes))
	}

	fn from_address(address: &str) -> Result<Self, NeoError> {
		let bytes = match bs58::decode(address).into_vec() {
			Ok(bytes) => bytes,
			Err(_) => return Err(NeoError::InvalidAddress),
		};
		let salt = bytes[0];
		let hash = &bytes[1..21];
		let checksum = &bytes[21..25];
		let mut sha = &bytes[..21].hash256().hash256();
		let check = &sha[..4];
		if checksum != check {
			return Err(NeoError::InvalidAddress)
			panic!("Invalid address checksum");
		}

		let mut rev = [0u8; 20];
		rev.clone_from_slice(hash);
		rev.reverse();
		Ok(Self::from_slice(&rev))
	}

	fn from_public_key(public_key: &PublicKey) -> Self {
		let script = ScriptBuilder::build_verification_script(public_key);
		Self::from_script(&script)
	}

	fn to_address(&self) -> String {
		let mut data = vec![DEFAULT_ADDRESS_VERSION];
		data.extend_from_slice(&self.0);
		let mut sha = &data.hash256().hash256();
		data.extend_from_slice(&sha[..4]);
		bs58::encode(data).into_string()
	}

	fn to_vec(&self) -> Vec<u8> {
		self.0.to_vec()
	}

	fn from_script(script: &[u8]) -> Self {
		let mut hash = script.sha256_ripemd160();
		hash.reverse();
		let mut arr = [0u8; 20];
		arr.copy_from_slice(&hash);
		Self(arr)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::types::public_key::PublicKeyExtension;
	use rustc_serialize::hex::ToHex;
	use std::str::FromStr;

	#[test]
	fn test_from_valid_hash() {
		assert_eq!(
			H160::from_hex("23ba2703c53263e8d6e522dc32203339dcd8eee9")
				.unwrap()
				.as_bytes()
				.to_hex(),
			"23ba2703c53263e8d6e522dc32203339dcd8eee9".to_string()
		);

		assert_eq!(
			H160::from_hex("0x23ba2703c53263e8d6e522dc32203339dcd8eee9")
				.unwrap()
				.as_bytes()
				.to_hex(),
			"23ba2703c53263e8d6e522dc32203339dcd8eee9".to_string()
		);
	}

	#[test]
	#[should_panic]
	fn test_creation_failures() {
		H160::from_hex("23ba2703c53263e8d6e522dc32203339dcd8eee").unwrap();
		H160::from_hex("g3ba2703c53263e8d6e522dc32203339dcd8eee9").unwrap();
		H160::from_hex("23ba2703c53263e8d6e522dc32203339dcd8ee").unwrap();
		H160::from_hex("c56f33fc6ecfcd0c225c4ab356fee59390af8560be0e930faebe74a6daff7c9b").unwrap();
	}

	#[test]
	fn test_to_array() {
		let hash = H160::from_str("23ba2703c53263e8d6e522dc32203339dcd8eee9").unwrap();
		assert_eq!(hash.to_vec(), hex::decode("23ba2703c53263e8d6e522dc32203339dcd8eee9").unwrap());
	}

	#[test]
	fn test_serialize_deserialize() {
		let expected = "23ba2703c53263e8d6e522dc32203339dcd8eee9";
		let expected_bytes: Vec<u8> = hex::decode(expected).unwrap().into_iter().rev().collect();

		// let mut writer = vec![];
		// // H160::from_str(expected).unwrap().serialize(&mut writer).unwrap();
		//
		// assert_eq!(writer, expected_bytes);
		// assert_eq!(H160::deserialize(&expected_bytes).unwrap().to_string(), expected);
	}

	#[test]
	fn test_equals() {
		let hash1 = H160::from_script(&hex::decode("01a402d8").unwrap());
		let hash2 = H160::from_script(&hex::decode("d802a401").unwrap());
		assert_ne!(hash1, hash2);
		assert_eq!(hash1, hash1);
	}

	#[test]
	fn test_from_address() {
		let hash = H160::from_address("NLnyLtep7jwyq1qhNPkwXbJpurC4jUT8ke").unwrap();
		let expected = hex::decode("09a55874c2da4b86e5d49ff530a1b153eb12c7d6").unwrap();
		assert_eq!(hash.to_le_vec(), expected);
	}

	#[test]
	// #[should_panic]
	fn test_from_invalid_address() {
		// assert that this should return Err
		assert_eq!(
			H160::from_address("NLnyLtep7jwyq1qhNPkwXbJpurC4jUT8keas"),
			Err(NeoError::InvalidAddress)
		);
	}

	#[test]
	fn test_from_public_key() {
		let key = "035fdb1d1f06759547020891ae97c729327853aeb1256b6fe0473bc2e9fa42ff50";
		let script = hex::decode(
			"0c21035fdb1d1f06759547020891ae97c729327853aeb1256b6fe0473bc2e9fa42ff504156e7b327",
		)
		.unwrap();
		let pubkey = PublicKey::from_hex(key).unwrap();
		let encode_key = pubkey.to_encoded_point(true).as_bytes().to_hex();
		assert_eq!(encode_key, key);

		let hash = H160::from_public_key(&pubkey);
		assert_eq!(hash.to_vec(), script.sha256_ripemd160());
	}

	#[test]
	fn test_from_contract_script() {
		let expected = hex::decode("0898ea2197378f623a7670974454448576d0aeaf").unwrap();
		let hash = H160::from_script(&hex::decode("110c21026aa8fe6b4360a67a530e23c08c6a72525afde34719c5436f9d3ced759f939a3d110b41138defaf").unwrap());
		assert_eq!(hash.to_vec(), expected);
	}

	#[test]
	fn test_to_address() {
		let pubkey =
			PublicKey::from_hex("250863ad64a87ae8a2fe83c1af1a8403cb53f53e486d8511dad8a04887e5b235")
				.unwrap();
		let hash = H160::from_public_key(&pubkey);
		assert_eq!(hash.to_address(), "AK2nJJpJr6o664CWJKi1QRXjqeic2zRp8y");
	}

	#[test]
	fn test_compare() {
		let hash1 = H160::from_script(&hex::decode("01a402d8").unwrap());
		let hash2 = H160::from_script(&hex::decode("d802a401").unwrap());
		let hash3 = H160::from_script(&hex::decode("a7b3a191").unwrap());

		assert!(hash2 > hash1);
		assert!(hash3 > hash1);
		assert!(hash2 > hash3);
	}

	#[test]
	fn test_size() {
		let hash = H160::from_str("23ba2703c53263e8d6e522dc32203339dcd8eee9").unwrap();
		assert_eq!(hash.as_bytes().len(), 20);
	}
}
