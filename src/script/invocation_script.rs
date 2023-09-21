// invocation_script

use crate::{
	crypto::{hash::HashableForVec, key_pair::KeyPair, sign::SignatureData},
	script::script_builder::ScriptBuilder,
	types::Bytes,
};
use p256::ecdsa::signature::Signer;

#[derive(Debug, Clone, PartialEq, Eq, Hash, CopyGetters, Setters)]
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
	derive_more::Display,
)]
pub struct InvocationScript {
	script: Bytes,
}

impl InvocationScript {
	// pub fn new() -> Self {
	// 	Self { 0:Bytes::new() }
	// }

	// pub fn from(script: Bytes) -> Self {
	// 	Self { 0 }
	// }

	pub fn from_signature(signature: &SignatureData) -> Self {
		let mut builder = ScriptBuilder::new()
			.push_data(signature.concatenated())
			.expect("TODO: panic message");
		Self { script: builder.into_bytes() }
	}

	pub fn from_message_and_key_pair(message: Bytes, key_pair: &KeyPair) -> Result<Self, ()> {
		let message_hash = message.hash256();
		let signature = key_pair.private_key().sign((&message_hash, key_pair).unwrap());
		let mut builder = ScriptBuilder::new();
		// Convert signature to bytes
		let mut signature_bytes = [0; 64];
		signature.write_scalars(&mut signature_bytes).unwrap();
		builder.push_data(signature_bytes.to_vec()).expect("Incorrect signature length");
		Ok(Self { script: builder.into_bytes() })
	}

	pub fn from_signatures(signatures: &[SignatureData]) -> Self {
		let mut builder = ScriptBuilder::new();
		for signature in signatures {
			let mut signature_bytes = [0; 64];
			signature.write_scalars(&mut signature_bytes).unwrap();

			builder.push_data(signature_bytes.to_vec()).expect("Incorrect signature length");
		}
		Self { script: builder.into_bytes() }
	}
}
