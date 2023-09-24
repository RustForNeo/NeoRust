use crate::{
	crypto::wif::{str_to_wif, Wif},
	neo_error::{NeoError, NeoError::InvalidPublicKey},
	types::{public_key::PublicKeyExtension, PrivateKey, PublicKey, ScriptHash},
};
use hex::FromHexError;
use rand::Rng;
use rustc_serialize::hex::ToHex;

pub trait PrivateKeyExtension
where
	Self: Sized,
{
	fn to_address(&self) -> String;

	fn to_script_hash(&self) -> ScriptHash;

	fn to_vec(&self) -> Vec<u8>;

	fn to_hex_str(&self) -> String;

	fn from_hex(hex: &str) -> Result<Self, hex::FromHexError>;

	fn from_slice(slice: &[u8]) -> Result<Self, NeoError>;

	fn from_wif(wif: &str) -> Result<Self, NeoError>;

	fn to_wif(&self) -> String;

	fn to_public_key(&self) -> PublicKey;
}

impl PrivateKeyExtension for PrivateKey {
	fn to_address(&self) -> String {
		PublicKey::from(self).to_address()
	}

	fn to_script_hash(&self) -> ScriptHash {
		todo!()
	}

	fn to_vec(&self) -> Vec<u8> {
		self.to_bytes().to_vec()
	}

	fn to_hex_str(&self) -> String {
		self.to_vec().to_hex()
	}

	fn from_hex(hex: &str) -> Result<Self, FromHexError> {
		let bytes = hex::decode(hex).unwrap();
		Ok(Self::from_slice(&bytes).unwrap())
	}

	fn from_slice(slice: &[u8]) -> Result<Self, NeoError> {
		if slice.len() != 32 {
			return Err(InvalidPublicKey)
		}

		let mut arr = [0u8; 32];
		arr.copy_from_slice(slice);
		Ok(Self::from_bytes(&arr).map_err(|_| InvalidPublicKey).unwrap())
	}

	fn from_wif(wif: &str) -> Result<Self, NeoError> {
		let bytes = str_to_wif(wif)?;
		Self::from_slice(&bytes)
	}

	fn to_wif(&self) -> String {
		self.to_vec().as_slice().to_wif()
	}

	fn to_public_key(&self) -> PublicKey {
		PublicKey::from(self)
	}
}
