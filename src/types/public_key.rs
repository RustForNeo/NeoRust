use crate::{
	neo_error::{NeoError, NeoError::InvalidPublicKey},
	types::{script_hash::ScriptHashExtension, PrivateKey, PublicKey, ScriptHash},
};
use hex::FromHexError;
use p256::{elliptic_curve::sec1::EncodedPoint, NistP256};
use primitive_types::H160;
use rustc_serialize::hex::ToHex;

pub trait PublicKeyExtension
where
	Self: Sized,
{
	fn to_address(&self) -> String;
	fn to_vec(&self) -> Vec<u8>;

	fn to_script_hash(&self) -> ScriptHash;

	fn to_hex_str(&self) -> String;

	fn from_slice(slice: &[u8]) -> Result<Self, NeoError>;
	fn from_hex(hex: &str) -> Result<Self, hex::FromHexError>;
	fn from_private_key(private_key: &PrivateKey) -> Self;
}
impl PublicKeyExtension for PublicKey {
	fn to_address(&self) -> String {
		H160::from_public_key(self).to_address()
	}

	fn to_vec(&self) -> Vec<u8> {
		self.to_encoded_point(false).as_bytes().to_vec()
	}

	fn to_script_hash(&self) -> ScriptHash {
		H160::from_public_key(self)
	}

	fn to_hex_str(&self) -> String {
		self.to_vec().as_slice().to_hex()
	}

	fn from_slice(slice: &[u8]) -> Result<Self, NeoError> {
		if slice.len() != 64 {
			return Err(InvalidPublicKey)
		}

		let mut arr = [0u8; 64];
		arr.copy_from_slice(slice);

		Ok(Self::from_encoded_point(&EncodedPoint::<NistP256>::from_bytes(slice).unwrap())
			.map_err(|_| InvalidPublicKey)
			.unwrap())
	}

	fn from_hex(hex: &str) -> Result<Self, FromHexError> {
		let bytes = hex::decode(hex).unwrap();
		Ok(Self::from_slice(&bytes).unwrap())
	}

	fn from_private_key(private_key: &PrivateKey) -> Self {
		PublicKey::from(private_key)
	}
}
