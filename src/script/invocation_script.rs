// invocation_script

use crate::{
	crypto::{key_pair::KeyPair, sign::SignatureData},
	script::script_builder::ScriptBuilder,
	types::Bytes,
};
use p256::ecdsa::Signature;

#[derive(Debug, Clone, PartialEq, Eq, Hash, CopyGetters, Setters, Default)]
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
	derive_more::IntoIterator,
	derive_more::Display,
)]
pub struct InvocationScript(Bytes);

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
		Self { 0: builder.into_bytes() }
	}

	pub fn from_message_and_key_pair(message: Bytes, key_pair: &KeyPair) -> Result<Self, ()> {
		let signature = Sign::sign_message(&message, key_pair)?;
		let mut builder = ScriptBuilder::new();
		builder.push_data(&signature.concatenated());
		Ok(Self { 0: builder.into_bytes() })
	}

	pub fn from_signatures(signatures: &[SignatureData]) -> Self {
		let mut builder = ScriptBuilder::new();
		for signature in signatures {
			builder.push_data(&signature.concatenated());
		}
		Self { 0: builder.into_bytes() }
	}
}
