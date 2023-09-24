use crate::{
	crypto::{hash::HashableForVec, wif::Wif},
	neo_error::NeoError,
	script::script_builder::ScriptBuilder,
	types::{script_hash::ScriptHashExtension, Address, PrivateKey, PublicKey, ScriptHash},
	utils::*,
};
use getset::{CopyGetters, Getters};
use p256::ecdsa::{signature::SignerMut, Signature, VerifyingKey};
use serde_derive::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize)]
pub struct KeyPair {
	#[getset(get = "pub", set = "pub")]
	#[serde(
		serialize_with = "serialize_private_key",
		deserialize_with = "deserialize_private_key"
	)]
	private_key: PrivateKey,
	#[getset(get = "pub", set = "pub")]
	#[serde(serialize_with = "serialize_public_key", deserialize_with = "deserialize_public_key")]
	public_key: PublicKey,
}

impl KeyPair {
	pub fn from_private_key(private_key: PrivateKey) -> Self {
		let public_key = VerifyingKey::from(&private_key);
		Self { private_key, public_key }
	}

	pub fn generate() -> Self {
		let mut rng = rand::thread_rng();
		let private_key = PrivateKey::random(&mut rng);
		Self::from_private_key(private_key)
	}

	pub fn get_address(&self) -> Result<Address, NeoError> {
		self.get_script_hash()
		// let address = script_hash.to_address();
		// Ok(address)
	}

	pub fn get_script_hash(&self) -> Result<ScriptHash, NeoError> {
		let script = ScriptBuilder::build_verification_script(&self.public_key);
		Ok(ScriptHash::from_script(&script))
	}

	pub fn sign(&mut self, message: &[u8]) -> Result<Signature, NeoError> {
		let message = message.hash256();
		let signature = self.private_key.sign(&message);
		Ok(signature)
	}

	pub fn export_wif(&self) -> String {
		self.private_key.to_bytes().as_slice().to_wif()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{crypto::wif::str_to_wif, types::private_key::PrivateKeyExtension};
	use p256::ecdsa::signature::Verifier;

	#[test]
	fn test_from_private_key() {
		let private_key = PrivateKey::random(&mut rand::thread_rng());
		let keypair = KeyPair::from_private_key(private_key);

		// assert_eq!(keypair.private_key, private_key);
		// assert_eq!(keypair.public_key, VerifyingKey::from(&private_key));
	}

	#[test]
	fn test_generate() {
		let keypair = KeyPair::generate();

		// assert!(keypair.private_key.is_valid());
		// assert!(keypair.public_key.is_valid());
	}

	#[test]
	fn test_get_address() {
		let keypair = KeyPair::generate();
		let address = keypair.get_address().unwrap();

		// assert!(address.is_valid());
	}

	#[test]
	fn test_get_script_hash() {
		let keypair = KeyPair::generate();
		let script_hash = keypair.get_script_hash().unwrap();

		let expected = ScriptHash::from_public_key(&keypair.public_key);
		assert_eq!(script_hash, expected);
	}

	#[test]
	fn test_sign() {
		let mut keypair = KeyPair::generate();
		let message = b"Hello World";
		let signature = keypair.sign(message).unwrap();

		assert!(keypair.public_key.verify(message, &signature).is_ok());
	}

	#[test]
	fn test_export_wif() {
		let keypair = KeyPair::generate();
		let wif = keypair.export_wif();

		assert_eq!(PrivateKey::from_wif(&wif).unwrap(), keypair.private_key);
	}
}
