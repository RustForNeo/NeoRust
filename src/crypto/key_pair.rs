use crate::{
	crypto::wif::Wif,
	neo_error::NeoError,
	script::script_builder::ScriptBuilder,
	types::{H160Externsion, PrivateKey},
};
use p256::{
	ecdsa::{signature::SignerMut, Signature},
	elliptic_curve::sec1::ToEncodedPoint,
	PublicKey,
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{error::Error, hash::Hash};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyPair {
	pub private_key: PrivateKey,
	pub public_key: PublicKey,
}

impl KeyPair {
	pub fn new(private_key: PrivateKey, public_key: PublicKey) -> Self {
		Self { private_key, public_key }
	}
	pub fn from_private_key(private_key: PrivateKey) -> Self {
		let public_key = p256::PublicKey::from_secret_key(&private_key);
		Self { private_key, public_key }
	}

	pub fn generate() -> Self {
		let mut rng = rand::thread_rng();
		let private_key = PrivateKey::random(&mut rng);
		Self::from_private_key(private_key)
	}

	pub fn get_address(&self) -> Result<String, NeoError> {
		let script_hash = self.get_script_hash()?;
		let address = script_hash.to_address();
		Ok(address)
	}

	pub fn get_script_hash(&self) -> Result<H160, NeoError> {
		let public_key = self.public_key.to_encoded_point(true);
		let script = ScriptBuilder::build_verification_script(&public_key)?;
		Ok(H160::from_script(&script)?)
	}

	pub fn sign(&mut self, message: &[u8]) -> Result<Signature, dyn Error> {
		let message = Sha256::digest(message);
		let signature = self.private_key.sign(&message)?;
		Ok(signature)
	}

	pub fn export_wif(&self) -> String {
		self.private_key.to_be_bytes().as_slice().to_wif()
	}
}

// Implementations for serialization

impl Serialize for KeyPair {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		use serde::ser::SerializeStruct;

		let mut strukt = serializer.serialize_struct("KeyPair", 2)?;
		strukt.serialize_field("private_key", &self.private_key)?;
		strukt.serialize_field("public_key", &self.public_key)?;
		strukt.end()
	}
}

impl<'de> Deserialize<'de> for KeyPair {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let kp = KeyPair::deserialize(deserializer)?;
		Ok(KeyPair { private_key: kp.private_key, public_key: kp.public_key })
	}
}
