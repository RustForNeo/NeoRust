use crate::{protocol::core::responses::neo_witness::NeoWitness, utils::*};
use primitive_types::H256;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NeoGetStateRoot {
	pub state_root: Option<StateRoot>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct StateRoot {
	pub version: u32,
	pub index: u32,
	#[serde(rename = "roothash")]
	#[serde(serialize_with = "serialize_h256")]
	#[serde(deserialize_with = "deserialize_h256")]
	pub root_hash: H256,
	pub witnesses: Vec<NeoWitness>,
}
