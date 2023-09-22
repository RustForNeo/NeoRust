use crate::{
	crypto::{key_pair::KeyPair, sign::SignatureData},
	neo_error::NeoError,
	script::{
		invocation_script::InvocationScript, script_builder::ScriptBuilder,
		verification_script::VerificationScript,
	},
	types::{contract_parameter::ContractParameter, Bytes, PublicKey},
};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use serde::{Deserialize, Serialize};

#[derive(Hash, Serialize, Deserialize, Debug, Clone)]
pub struct Witness {
	pub invocation_script: InvocationScript,
	pub verification_script: VerificationScript,
}

impl Witness {
	pub fn new() -> Self {
		Self {
			invocation_script: InvocationScript::new(),
			verification_script: VerificationScript::new(),
		}
	}

	pub fn from_scripts(invocation_script: Bytes, verification_script: Bytes) -> Self {
		Self {
			invocation_script: InvocationScript::from(invocation_script),
			verification_script: VerificationScript::from(verification_script),
		}
	}

	pub fn from_scripts_obj(
		invocation_script: InvocationScript,
		verification_script: VerificationScript,
	) -> Self {
		Self { invocation_script, verification_script }
	}

	pub async fn create(message_to_sign: Bytes, key_pair: &KeyPair) -> Result<Self, NeoError> {
		let invocation_script =
			InvocationScript::from_message_and_key_pair(message_to_sign, key_pair)
				.await
				.unwrap();
		let verification_script = VerificationScript::from(
			key_pair.public_key().to_encoded_point(false).as_bytes().to_vec(),
		);
		Ok(Self { invocation_script, verification_script })
	}

	pub async fn create_multisig_witness(
		signing_threshold: u8,
		signatures: Vec<SignatureData>,
		public_keys: Vec<PublicKey>,
	) -> Result<Self, NeoError> {
		let verification_script =
			VerificationScript::from_multisig(&public_keys, signing_threshold).await;
		Self::create_multisig_witness_script(signatures, verification_script).await
	}

	pub async fn create_multisig_witness_script(
		signatures: Vec<SignatureData>,
		verification_script: VerificationScript,
	) -> Result<Self, NeoError> {
		let threshold = verification_script.get_signing_threshold().await;
		if signatures.len() < threshold as usize {
			return Err(NeoError::IllegalArgument(
				"Not enough signatures provided for the required signing threshold.".to_string(),
			))
		}

		let invocation_script =
			InvocationScript::from_signatures(&signatures[..threshold as usize]).await;
		Ok(Self { invocation_script, verification_script })
	}

	pub async fn create_contract_witness(params: Vec<ContractParameter>) -> Result<Self, NeoError> {
		if params.is_empty() {
			return Ok(Self::new())
		}

		let mut builder = ScriptBuilder::new();
		for param in params {
			builder.push_param(&param).await.expect("Failed to push param");
		}
		let invocation_script = builder.to_bytes();

		Ok(Self {
			invocation_script: InvocationScript::from(invocation_script),
			verification_script: VerificationScript::new(),
		})
	}
}
