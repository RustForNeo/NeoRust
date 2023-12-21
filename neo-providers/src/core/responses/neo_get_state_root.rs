use crate::core::transaction::witness::Witness;
use neo_types::*;
use primitive_types::H256;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct StateRoot {
	pub version: u32,
	pub index: u32,
	#[serde(rename = "roothash")]
	#[serde(serialize_with = "serialize_h256")]
	#[serde(deserialize_with = "deserialize_h256")]
	pub root_hash: H256,
	pub witnesses: Vec<Witness>,
}
