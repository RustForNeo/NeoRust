use crate::{
	crypto::{hash::HashableForVec, wif::Wif},
	neo_error::NeoError,
	script::script_builder::ScriptBuilder,
	types::{Address, H160Externsion, PrivateKey, PublicKey},
	utils::*,
};
use getset::{CopyGetters, Getters};
use p256::ecdsa::{signature::SignerMut, Signature, VerifyingKey};
use primitive_types::H160;
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
	// pub fn new(private_key: PrivateKey, public_key: PublicKey) -> Self {
	// 	Self { private_key, public_key }
	// }

	pub fn from_private_key(private_key: PrivateKey) -> Self {
		let public_key = VerifyingKey::from(&private_key); //. p256::PublicKey::from_secret_key(&private_key);
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

	pub fn get_script_hash(&self) -> Result<H160, NeoError> {
		let script = ScriptBuilder::build_verification_script(&self.public_key);
		Ok(H160::from_script(&script))
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
