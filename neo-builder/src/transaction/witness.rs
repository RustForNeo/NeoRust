use crate::{
	contract_parameter::ContractParameter, invocation_script::InvocationScript,
	transaction_error::TransactionError, verification_script::VerificationScript, Bytes,
};
use neo_crypto::{key_pair::KeyPair, signature::Signature};
use p256::{elliptic_curve::sec1::ToEncodedPoint, PublicKey};
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

	pub fn create(message_to_sign: Bytes, key_pair: &KeyPair) -> Result<Self, TransactionError> {
		let invocation_script =
			InvocationScript::from_message_and_key_pair(message_to_sign, key_pair).unwrap();
		let verification_script = VerificationScript::from(
			key_pair.public_key().to_encoded_point(false).as_bytes().to_vec(),
		);
		Ok(Self { invocation: invocation_script, verification: verification_script })
	}

	pub fn create_multisig_witness(
		signing_threshold: u8,
		signatures: Vec<Signature>,
		public_keys: Vec<PublicKey>,
	) -> Result<Self, TransactionError> {
		let verification_script =
			VerificationScript::from_multisig(&public_keys, signing_threshold);
		Self::create_multisig_witness_script(signatures, verification_script)
	}

	pub fn create_multisig_witness_script(
		signatures: Vec<Signature>,
		verification_script: VerificationScript,
	) -> Result<Self, TransactionError> {
		let threshold = verification_script.get_signing_threshold().unwrap();
		if signatures.len() < threshold {
			return Err(TransactionError::SignerConfiguration(
				"Not enough signatures provided for the required signing threshold.".to_string(),
			))
		}

		let invocation_script =
			InvocationScript::from_signatures(&signatures[..threshold as usize]);
		Ok(Self { invocation: invocation_script, verification: verification_script })
	}

	pub fn create_contract_witness(
		params: Vec<ContractParameter>,
	) -> Result<Self, TransactionError> {
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
