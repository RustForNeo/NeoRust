use neo_types::*;

use neo_crypto::keys::Secp256r1PublicKey;
use primitive_types::H160;
use serde::{Deserialize, Deserializer, Serialize};
use std::hash::{Hash, Hasher};

/// Enum representing the different types of witness conditions that can be used in a smart contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WitnessCondition {
	/// Boolean value.
	Boolean(bool),
	/// Not operator.
	Not(Box<WitnessCondition>),
	/// And operator.
	And(Vec<WitnessCondition>),
	/// Or operator.
	Or(Vec<WitnessCondition>),
	/// Script hash.
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	ScriptHash(H160),
	/// Public key group.
	#[serde(deserialize_with = "deserialize_public_key")]
	#[serde(serialize_with = "serialize_public_key")]
	Group(Secp256r1PublicKey),
	/// Called by entry.
	CalledByEntry,
	/// Called by contract.
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	CalledByContract(H160),
	/// Called by public key group.
	#[serde(deserialize_with = "deserialize_public_key")]
	#[serde(serialize_with = "serialize_public_key")]
	CalledByGroup(Secp256r1PublicKey),
}

impl Hash for WitnessCondition {
	/// Hashes the witness condition.
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self {
			WitnessCondition::Boolean(b) => b.hash(state),
			WitnessCondition::Not(exp) => exp.hash(state),
			WitnessCondition::And(exp) => exp.hash(state),
			WitnessCondition::Or(exp) => exp.hash(state),
			WitnessCondition::ScriptHash(hash) => hash.hash(state),
			WitnessCondition::Group(group) => group.to_raw_bytes().to_vec().hash(state),
			WitnessCondition::CalledByEntry => WitnessCondition::CalledByEntry.hash(state),
			WitnessCondition::CalledByContract(hash) => hash.hash(state),
			WitnessCondition::CalledByGroup(group) => group.to_raw_bytes().to_vec().hash(state),
		}
	}
}

impl WitnessCondition {
	/// Maximum number of subitems.
	const MAX_SUBITEMS: usize = 16;
	/// Maximum nesting depth.
	const MAX_NESTING_DEPTH: usize = 2;

	/// Boolean value string.
	const BOOLEAN_VALUE: &'static str = "Boolean";
	/// Not operator string.
	const NOT_VALUE: &'static str = "Not";
	/// And operator string.
	const AND_VALUE: &'static str = "And";
	/// Or operator string.
	const OR_VALUE: &'static str = "Or";
	/// Script hash string.
	const SCRIPT_HASH_VALUE: &'static str = "ScriptHash";
	/// Public key group string.
	const GROUP_VALUE: &'static str = "Group";
	/// Called by entry string.
	const CALLED_BY_ENTRY_VALUE: &'static str = "CalledByEntry";
	/// Called by contract string.
	const CALLED_BY_CONTRACT_VALUE: &'static str = "CalledByContract";
	/// Called by public key group string.
	const CALLED_BY_GROUP_VALUE: &'static str = "CalledByGroup";

	/// Boolean byte value.
	const BOOLEAN_BYTE: u8 = 0x00;
	/// Not operator byte value.
	const NOT_BYTE: u8 = 0x01;
	/// And operator byte value.
	const AND_BYTE: u8 = 0x02;
	/// Or operator byte value.
	const OR_BYTE: u8 = 0x03;
	/// Script hash byte value.
	const SCRIPT_HASH_BYTE: u8 = 0x18;
	/// Public key group byte value.
	const GROUP_BYTE: u8 = 0x19;
	/// Called by entry byte value.
	const CALLED_BY_ENTRY_BYTE: u8 = 0x20;
	/// Called by contract byte value.
	const CALLED_BY_CONTRACT_BYTE: u8 = 0x28;
	/// Called by public key group byte value.
	const CALLED_BY_GROUP_BYTE: u8 = 0x29;

	/// Returns the JSON value of the witness condition.
	pub fn json_value(&self) -> &'static str {
		match self {
			WitnessCondition::Boolean(_) => WitnessCondition::BOOLEAN_VALUE,
			WitnessCondition::Not(_) => WitnessCondition::NOT_VALUE,
			WitnessCondition::And(_) => WitnessCondition::AND_VALUE,
			WitnessCondition::Or(_) => WitnessCondition::OR_VALUE,
			WitnessCondition::ScriptHash(_) => WitnessCondition::SCRIPT_HASH_VALUE,
			WitnessCondition::Group(_) => WitnessCondition::GROUP_VALUE,
			WitnessCondition::CalledByEntry => WitnessCondition::CALLED_BY_ENTRY_VALUE,
			WitnessCondition::CalledByContract(_) => WitnessCondition::CALLED_BY_CONTRACT_VALUE,
			WitnessCondition::CalledByGroup(_) => WitnessCondition::CALLED_BY_GROUP_VALUE,
		}
	}

	/// Returns the byte value of the witness condition.
	pub fn byte(&self) -> u8 {
		match self {
			WitnessCondition::Boolean(_) => WitnessCondition::BOOLEAN_BYTE,
			WitnessCondition::Not(_) => WitnessCondition::NOT_BYTE,
			WitnessCondition::And(_) => WitnessCondition::AND_BYTE,
			WitnessCondition::Or(_) => WitnessCondition::OR_BYTE,
			WitnessCondition::ScriptHash(_) => WitnessCondition::SCRIPT_HASH_BYTE,
			WitnessCondition::Group(_) => WitnessCondition::GROUP_BYTE,
			WitnessCondition::CalledByEntry => WitnessCondition::CALLED_BY_ENTRY_BYTE,
			WitnessCondition::CalledByContract(_) => WitnessCondition::CALLED_BY_CONTRACT_BYTE,
			WitnessCondition::CalledByGroup(_) => WitnessCondition::CALLED_BY_GROUP_BYTE,
		}
	}

	/// Returns the boolean expression of the witness condition.
	pub fn boolean_expression(&self) -> Option<bool> {
		match self {
			WitnessCondition::Boolean(b) => Some(*b),
			_ => None,
		}
	}

	/// Returns the expression of the witness condition.
	pub fn expression(&self) -> Option<&WitnessCondition> {
		match self {
			WitnessCondition::Not(exp) => Some(&exp),
			_ => None,
		}
	}

	/// Returns the expression list of the witness condition.
	pub fn expression_list(&self) -> Option<&[WitnessCondition]> {
		match self {
			WitnessCondition::And(exp) | WitnessCondition::Or(exp) => Some(&exp),
			_ => None,
		}
	}

	/// Returns the script hash of the witness condition.
	pub fn script_hash(&self) -> Option<&H160> {
		match self {
			WitnessCondition::ScriptHash(hash) | WitnessCondition::CalledByContract(hash) =>
				Some(hash),
			_ => None,
		}
	}

	/// Returns the public key group of the witness condition.
	pub fn group(&self) -> Option<&Secp256r1PublicKey> {
		match self {
			WitnessCondition::Group(group) | WitnessCondition::CalledByGroup(group) => Some(group),
			_ => None,
		}
	}
}
