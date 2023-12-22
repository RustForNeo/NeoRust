use crate::core::{
	error::BuilderError,
	script::script_builder::ScriptBuilder,
	transaction::{invocation_script::InvocationScript, verification_script::VerificationScript},
};
use neo_codec::{encode::NeoSerializable, Decoder, Encoder};
use neo_crypto::{
	key_pair::KeyPair,
	keys::{Secp256r1PublicKey, Secp256r1Signature},
};
use neo_types::{contract_parameter::ContractParameter, Bytes};
use serde::{Deserialize, Serialize};

#[derive(Hash, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Witness {
	pub invocation: InvocationScript,
	pub verification: VerificationScript,
}

impl Witness {
	pub fn new() -> Self {
		Self { invocation: InvocationScript::new(), verification: VerificationScript::new() }
	}

	pub fn from_scripts(invocation_script: Bytes, verification_script: Bytes) -> Self {
		Self {
			invocation: InvocationScript::from(invocation_script),
			verification: VerificationScript::from(verification_script),
		}
	}

	pub fn from_scripts_obj(
		invocation_script: InvocationScript,
		verification_script: VerificationScript,
	) -> Self {
		Self { invocation: invocation_script, verification: verification_script }
	}

	pub fn create(message_to_sign: Bytes, key_pair: &mut KeyPair) -> Result<Self, BuilderError> {
		let invocation_script =
			InvocationScript::from_message_and_key_pair(message_to_sign, key_pair).unwrap();
		let verification_script =
			VerificationScript::from(key_pair.public_key().to_raw_bytes().to_vec());
		Ok(Self { invocation: invocation_script, verification: verification_script })
	}

	pub fn create_multi_sig_witness(
		signing_threshold: u8,
		signatures: Vec<Secp256r1Signature>,
		public_keys: Vec<Secp256r1PublicKey>,
	) -> Result<Self, BuilderError> {
		let verification_script =
			VerificationScript::from_multi_sig(&public_keys, signing_threshold);
		Self::create_multi_sig_witness_script(signatures, verification_script)
	}

	pub fn create_multi_sig_witness_script(
		signatures: Vec<Secp256r1Signature>,
		verification_script: VerificationScript,
	) -> Result<Self, BuilderError> {
		let threshold = verification_script.get_signing_threshold().unwrap();
		if signatures.len() < threshold {
			return Err(BuilderError::SignerConfiguration(
				"Not enough signatures provided for the required signing threshold.".to_string(),
			))
		}

		let invocation_script =
			InvocationScript::from_signatures(&signatures[..threshold as usize]);
		Ok(Self { invocation: invocation_script, verification: verification_script })
	}

	pub fn create_contract_witness(params: Vec<ContractParameter>) -> Result<Self, BuilderError> {
		if params.is_empty() {
			return Ok(Self::new())
		}

		let mut builder = ScriptBuilder::new();
		for param in params {
			builder.push_param(&param).expect("Failed to push param");
		}
		let invocation_script = builder.to_bytes();

		Ok(Self {
			invocation: InvocationScript::from(invocation_script),
			verification: VerificationScript::new(),
		})
	}
}

impl NeoSerializable for Witness {
	type Error = BuilderError;

	fn size(&self) -> usize {
		self.invocation.size() + self.verification.size()
	}

	fn encode(&self, writer: &mut Encoder) {
		self.invocation.encode(writer);
		self.verification.encode(writer);
	}

	fn decode(reader: &mut Decoder) -> Result<Self, Self::Error> {
		let invocation = InvocationScript::decode(reader)?;
		let verification = VerificationScript::decode(reader)?;
		Ok(Self { invocation, verification })
	}
	fn to_array(&self) -> Vec<u8> {
		let mut writer = Encoder::new();
		self.encode(&mut writer);
		writer.to_bytes()
	}
}
