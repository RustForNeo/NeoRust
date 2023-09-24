use crate::{crypto::hash::HashableForVec, neo_error::NeoError, types::PublicKey};
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
	fn to_address(&self) -> String;
	fn to_vec(&self) -> Vec<u8>;
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
		let bytes = hex::decode(hex).unwrap();
		Ok(Self::from_slice(&bytes))
	}

	fn from_address(address: &str) -> Result<Self, NeoError> {
		let bytes = bs58::decode(address).into_vec().unwrap();

		Ok(Self::from_slice(&bytes))
	}

	fn from_public_key(public_key: &PublicKey) -> Self {
		let hash = public_key.to_encoded_point(false).as_bytes().sha256_ripemd160();

		let mut arr = [0u8; 20];
		arr.copy_from_slice(&hash);
		Self(arr)
	}

	fn to_address(&self) -> String {
		bs58::encode(&self.0).into_string()
	}

	fn to_vec(&self) -> Vec<u8> {
		self.0.to_vec()
	}

	fn from_script(script: &[u8]) -> Self {
		let result = script.sha256_ripemd160();
		let mut arr = [0u8; 20];
		arr.copy_from_slice(&result);
		Self(arr)
	}
}
