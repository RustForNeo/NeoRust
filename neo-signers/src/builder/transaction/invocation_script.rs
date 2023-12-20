use crate::script::script_builder::ScriptBuilder;
use getset::{Getters, Setters};
use neo_crypto::{hash::HashableForVec, key_pair::KeyPair, signature::Signature};
use neo_types::Bytes;
use p256::ecdsa::signature::Signer;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Getters, Setters)]
#[getset(get_copy, set)]
#[derive(educe::Educe)]
// note `new` below: generate `new()` that calls Default
#[educe(Default(new))]
#[derive(
	derive_more::AsRef,
	derive_more::Deref,
	derive_more::IntoIterator,
	derive_more::Index,
	derive_more::IndexMut,
	derive_more::Into,
	derive_more::From,
)]
pub struct InvocationScript {
	script: Bytes,
}

impl InvocationScript {
	pub fn from_signature(signature: &Signature) -> Self {
		let mut builder = ScriptBuilder::new();
		builder.push_data(signature.concatenated()).expect("TODO: panic message");
		Self { script: builder.to_bytes() }
	}

	pub fn from_message_and_key_pair(message: Bytes, key_pair: &KeyPair) -> Result<Self, ()> {
		let message_hash = message.hash256();
		let signature = key_pair.private_key().sign(&message_hash);
		let mut builder = ScriptBuilder::new();
		// Convert signature to bytes
		let signature_bytes = signature.to_vec();
		builder.push_data(signature_bytes).expect("Incorrect signature length");
		Ok(Self { script: builder.to_bytes() })
	}

	pub fn from_signatures(signatures: &[Signature]) -> Self {
		let mut builder = ScriptBuilder::new();
		for signature in signatures {
			let mut signature_bytes = signature.concatenated();
			// signature.write_scalars(&mut signature_bytes).unwrap();

			builder.push_data(signature_bytes.to_vec()).expect("Incorrect signature length");
		}
		Self { script: builder.to_bytes() }
	}
}
