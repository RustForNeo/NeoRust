use crate::{
	neo_error::NeoError,
	script::{interop_service::InteropService, op_code::OpCode, script_builder::ScriptBuilder},
	serialization::binary_reader::BinaryReader,
	types::{Bytes, PublicKey, PublicKeyExtension},
};
use p256::{ecdsa::Signature, pkcs8::der::Encode};
use primitive_types::H160;
use serde_derive::{Deserialize, Serialize};
use std::vec;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VerificationScript {
	script: Bytes,
}

impl VerificationScript {
	pub fn new() -> Self {
		Self { script: Bytes::new() }
	}

	pub fn from(script: Bytes) -> Self {
		Self { script: script.to_vec().unwrap() }
	}

	pub async fn from_public_key(public_key: &PublicKey) -> Self {
		let mut builder = ScriptBuilder::new();
		builder
			.push_data(public_key.to_encoded_point(false).as_bytes().to_vec())
			.await
			.unwrap()
			.op_code(&vec![OpCode::Syscall])
			.await
			.push_data(InteropService::SystemCryptoCheckSig.hash().into_bytes())
			.await
			.unwrap();
		Self::from(builder.to_bytes())
	}

	pub async fn from_multisig(public_keys: &[PublicKey], threshold: u8) -> Self {
		// Build multi-sig script
		let mut builder = ScriptBuilder::new();
		builder
			.push_integer(threshold as i64)
			.await
			.expect("Threshold must be between 1 and 16");
		for key in public_keys {
			builder.push_data(key.to_vec()).await.unwrap();
		}
		let a = builder
			.push_integer(public_keys.len() as i64)
			.await
			.unwrap()
			.op_code(vec![OpCode::Syscall].as_slice())
			.await
			.push_data(InteropService::SystemCryptoCheckMultisig.hash().into_bytes())
			.await
			.unwrap();
		Self::from(builder.to_bytes())
	}

	pub fn is_single_sig(&self) -> bool {
		self.script.len() == 35
			&& self.script[0] == OpCode::PushData1 as u8
			&& self.script[34] == OpCode::Syscall as u8
	}

	pub fn is_multisig(&self) -> bool {
		if self.script.len() < 37 {
			return false
		}

		let mut reader = BinaryReader::new(&self.script);

		let n = reader.read_var_int().unwrap();
		if !(1..16).contains(&n) {
			return false
		}

		let mut m = 0;
		while reader.read_u8() == OpCode::PushData1 as u8 {
			let len = reader.read_u8();
			if len != 33 {
				return false
			}
			let _ = reader.skip(33);
			m += 1;
		}

		if !(m >= n && m <= 16) {
			return false
		}

		// additional checks
		let service_bytes = &self.script[self.script.len() - 4..];
		if service_bytes != &InteropService::SystemCryptoCheckMultisig.hash().into_bytes() {
			return false
		}

		if m != reader.read_var_int().unwrap() {
			return false
		}

		if reader.read_u8() != OpCode::Syscall as u8 {
			return false
		}

		true
	}

	// other methods
	pub fn hash(&self) -> H160 {
		H160::from_slice(&self.script)
	}

	pub fn get_signatures(&self) -> Vec<Signature> {
		let mut reader = BinaryReader::new(&self.script);
		let mut signatures = vec![];

		while reader.read_u8() == OpCode::PushData1 as u8 {
			let len = reader.read_u8();
			let sig = Signature::from_der(&reader.read_bytes(len as usize).unwrap()).unwrap();
			signatures.push(sig);
		}

		signatures
	}

	pub fn get_public_keys(&self) -> Result<Vec<PublicKey>, NeoError> {
		if self.is_single_sig() {
			let mut reader = BinaryReader::new(&self.script);
			reader.read_u8(); // skip pushdata1
			reader.read_u8(); // skip length

			let mut point = [0; 33];
			point.copy_from_slice(&reader.read_bytes(33).unwrap());

			let key = PublicKey::from_sec1_bytes(&point).unwrap();
			return Ok(vec![key])
		}

		if self.is_multisig() {
			let mut reader = BinaryReader::new(&self.script);
			reader.read_var_int().unwrap(); // skip threshold

			let mut keys = vec![];
			while reader.read_u8() == OpCode::PushData1 as u8 {
				reader.read_u8(); // skip length
				let mut point = [0; 33];
				point.copy_from_slice(&reader.read_bytes(33).unwrap());
				keys.push(PublicKey::from_sec1_bytes(&point).unwrap());
			}

			return Ok(keys)
		}

		Err(NeoError::InvalidScript("Invalid verification script".to_string()))
	}
}
