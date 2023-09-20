use crate::{
	crypto::wif::Wif,
	neo_error::NeoError,
	script::script_builder::ScriptBuilder,
	types::{H160Externsion, PrivateKey, PublicKey},
};
use getset::{CopyGetters, Getters};
use p256::{
	ecdsa::{signature::SignerMut, Signature},
	elliptic_curve::sec1::ToEncodedPoint,
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{error::Error, hash::Hash};

#[derive(
	Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters, CopyGetters, educe::Educe,
)]
#[educe(Default(new))]
pub struct KeyPair {
	#[getset(get = "pub", set = "pub")]
	private_key: PrivateKey,
	#[getset(get = "pub", set = "pub")]
	public_key: PublicKey,
}

impl KeyPair {
	// pub fn new(private_key: PrivateKey, public_key: PublicKey) -> Self {
	// 	Self { private_key, public_key }
	// }

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
		let public_key = self.public_key.to_encoded_point(false);
		let script = ScriptBuilder::build_verification_script(&public_key)?;
		Ok(H160::from_script(&script)?)
	}

	pub fn sign(&mut self, message: &[u8]) -> Result<Signature, NeoError> {
		let message = Sha256::digest(message);
		let signature = self.private_key.sign(&message)?;
		Ok(signature)
	}

	pub fn export_wif(&self) -> String {
		self.private_key.to_be_bytes().as_slice().to_wif()
	}
}
