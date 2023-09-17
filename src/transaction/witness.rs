use crate::{
	crypto::{key_pair::KeyPair, sign::SignatureData},
	neo_error::NeoError,
	script::{
		invocation_script::InvocationScript, script_builder::ScriptBuilder,
		verification_script::VerificationScript,
	},
	types::{contract_parameter::ContractParameter, Bytes},
};
use p256::{elliptic_curve::sec1::ToEncodedPoint, PublicKey};
use serde::{Deserialize, Serialize};

#[derive(Hash)]
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

	pub fn create(message_to_sign: Bytes, key_pair: &KeyPair) -> Result<Self, NeoError> {
		let invocation_script =
			InvocationScript::from_message_and_key_pair(message_to_sign, key_pair)?;
		let verification_script = VerificationScript::from(
			key_pair.public_key.to_encoded_point(false).as_bytes().to_vec(),
		);
		Ok(Self { invocation_script, verification_script })
	}

	pub fn create_multisig_witness(
		signing_threshold: u8,
		signatures: Vec<SignatureData>,
		public_keys: Vec<PublicKey>,
	) -> Result<Self, NeoError> {
		let verification_script = VerificationScript::multisig(public_keys, signing_threshold)?;
		Self::create_multisig_witness_script(signatures, verification_script)
	}

	pub fn create_multisig_witness_script(
		signatures: Vec<SignatureData>,
		verification_script: VerificationScript,
	) -> Result<Self, NeoError> {
		let threshold = verification_script.get_signing_threshold()?;
		if signatures.len() < threshold as usize {
			return Err(NeoError::IllegalArgument(
				"Not enough signatures provided for the required signing threshold.".to_string(),
			));
		}

		let invocation_script =
			InvocationScript::from_signatures(&signatures[..threshold as usize]);
		Ok(Self { invocation_script, verification_script })
	}

	pub fn create_contract_witness(params: Vec<ContractParameter>) -> Result<Self, NeoError> {
		if params.is_empty() {
			return Ok(Self::new());
		}

		let mut builder = ScriptBuilder::new();
		for param in params {
			builder.push_param(&Some(param))?;
		}
		let invocation_script = builder.into_bytes();

		Ok(Self { invocation_script, verification_script: VerificationScript::new() })
	}
}
impl Serialize for Witness {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		use serde::ser::SerializeStruct;

		let mut strukt = serializer.serialize_struct("Witness", 2)?;
		strukt.serialize_field("invocation_script", &self.invocation_script)?;
		strukt.serialize_field("verification_script", &self.verification_script)?;
		strukt.end()
	}
}

impl<'de> Deserialize<'de> for Witness {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct WitnessVisitor;

		impl<'de> serde::de::Visitor<'de> for WitnessVisitor {
			type Value = Witness;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("struct Witness")
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::SeqAccess<'de>,
			{
				let invocation_script =
					seq.next_element()?.ok_or(serde::de::Error::invalid_length(0, &self))?;
				let verification_script =
					seq.next_element()?.ok_or(serde::de::Error::invalid_length(1, &self))?;

				Ok(Witness { invocation_script, verification_script })
			}
		}

		deserializer.deserialize_struct(
			"Witness",
			&["invocation_script", "verification_script"],
			WitnessVisitor,
		)
	}
}
