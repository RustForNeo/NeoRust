use crate::types::PublicKey;
use primitive_types::H160;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Hash, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WitnessCondition {
	Boolean(bool),
	Not(Box<WitnessCondition>),
	And(Vec<WitnessCondition>),
	Or(Vec<WitnessCondition>),
	ScriptHash(H160),
	Group(PublicKey),
	CalledByEntry,
	CalledByContract(H160),
	CalledByGroup(PublicKey),
}

impl WitnessCondition {
	const MAX_SUBITEMS: usize = 16;
	const MAX_NESTING_DEPTH: usize = 2;

	const BOOLEAN_VALUE: &'static str = "Boolean";
	const NOT_VALUE: &'static str = "Not";
	const AND_VALUE: &'static str = "And";
	const OR_VALUE: &'static str = "Or";
	const SCRIPT_HASH_VALUE: &'static str = "ScriptHash";
	const GROUP_VALUE: &'static str = "Group";
	const CALLED_BY_ENTRY_VALUE: &'static str = "CalledByEntry";
	const CALLED_BY_CONTRACT_VALUE: &'static str = "CalledByContract";
	const CALLED_BY_GROUP_VALUE: &'static str = "CalledByGroup";

	const BOOLEAN_BYTE: u8 = 0x00;
	const NOT_BYTE: u8 = 0x01;
	const AND_BYTE: u8 = 0x02;
	const OR_BYTE: u8 = 0x03;
	const SCRIPT_HASH_BYTE: u8 = 0x18;
	const GROUP_BYTE: u8 = 0x19;
	const CALLED_BY_ENTRY_BYTE: u8 = 0x20;
	const CALLED_BY_CONTRACT_BYTE: u8 = 0x28;
	const CALLED_BY_GROUP_BYTE: u8 = 0x29;

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

	// other methods

	pub fn boolean_expression(&self) -> Option<bool> {
		match self {
			WitnessCondition::Boolean(b) => Some(*b),
			_ => None,
		}
	}

	pub fn expression(&self) -> Option<&WitnessCondition> {
		match self {
			WitnessCondition::Not(exp) => Some(&exp),
			_ => None,
		}
	}

	pub fn expression_list(&self) -> Option<&[WitnessCondition]> {
		match self {
			WitnessCondition::And(exp) | WitnessCondition::Or(exp) => Some(&exp),
			_ => None,
		}
	}

	pub fn script_hash(&self) -> Option<&H160> {
		match self {
			WitnessCondition::ScriptHash(hash) | WitnessCondition::CalledByContract(hash) =>
				Some(hash),
			_ => None,
		}
	}

	pub fn group(&self) -> Option<&PublicKey> {
		match self {
			WitnessCondition::Group(group) | WitnessCondition::CalledByGroup(group) => Some(group),
			_ => None,
		}
	}
}
// Serialization
