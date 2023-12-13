use crate::{script::script_builder::ScriptBuilder, transaction::{transaction_attribute::TransactionAttribute, transaction_send_token::TransactionSendToken, signers::{signer::Signer, transaction_signer::TransactionSigner}}};
use hex::FromHexError;
use neo_types::{serde_value::ValueExtension, contract_parameter::ContractParameter};
use p256::PublicKey;
use primitive_types::H160;
use serde_json::Value;

pub type ScriptHash = H160;

/// Converts a list of public keys to a script hash using a given threshold.
///
/// # Arguments
///
/// * `public_keys` - A mutable slice of `PublicKey` instances.
/// * `threshold` - The minimum number of signatures required to validate the transaction.
///
/// # Returns
///
/// A `ScriptHash` instance representing the script hash of the multisig script.
pub fn public_keys_to_scripthash(public_keys: &mut [PublicKey], threshold: usize) -> ScriptHash {
	let mut script = ScriptBuilder::build_multisig_script(public_keys, threshold as u8).unwrap();
	// Self::from_script(&script)
	ScriptHash::from_slice(&script)
}

/// Converts a public key to a script hash.
///
/// # Arguments
///
/// * `public_key` - A `PublicKey` instance.
///
/// # Returns
///
/// A `ScriptHash` instance representing the script hash of the verification script.
pub fn pubkey_to_scripthash(public_key: &PublicKey) -> ScriptHash {
	let script = ScriptBuilder::build_verification_script(public_key);
	ScriptHash::from_script(&script)
}

impl ValueExtension for TransactionAttribute {
	fn to_value(&self) -> Value {
		Value::String(self.to_json())
	}
}

impl ValueExtension for TransactionSendToken {
	fn to_value(&self) -> Value {
		Value::String(serde_json::to_string(self).unwrap())
	}
}

impl ValueExtension for Vec<TransactionSendToken> {
	fn to_value(&self) -> Value {
		self.iter().map(|x| x.to_value()).collect()
	}
}
impl ValueExtension for Vec<TransactionAttribute> {
	fn to_value(&self) -> Value {
		self.iter().map(|x| x.to_value()).collect()
	}
}
impl ValueExtension for Signer {
	fn to_value(&self) -> Value {
		Value::String(serde_json::to_string(self).unwrap())
	}
}

impl ValueExtension for Vec<Signer> {
	fn to_value(&self) -> Value {
		self.iter().map(|x| x.to_value()).collect()
	}
}

impl ValueExtension for TransactionSigner {
	fn to_value(&self) -> Value {
		Value::String(serde_json::to_string(self).unwrap())
	}
}

impl ValueExtension for Vec<TransactionSigner> {
	fn to_value(&self) -> Value {
		self.iter().map(|x| x.to_value()).collect()
	}
}

impl ValueExtension for ContractParameter {
	fn to_value(&self) -> Value {
		Value::String(serde_json::to_string(self).unwrap())
	}
}

impl ValueExtension for Vec<ContractParameter> {
	fn to_value(&self) -> Value {
		self.iter().map(|x| x.to_value()).collect()
	}
}
