use crate::{
	protocol::core::responses::{neo_witness::NeoWitness, transaction::Transaction},
	utils::*,
};
use primitive_types::H256;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NeoBlock {
	#[serde(serialize_with = "serialize_h256")]
	#[serde(deserialize_with = "deserialize_h256")]
	pub hash: H256,
	pub size: u32,
	pub version: u32,
	#[serde(rename = "previousblockhash")]
	#[serde(serialize_with = "serialize_h256")]
	#[serde(deserialize_with = "deserialize_h256")]
	pub prev_block_hash: H256,
	#[serde(rename = "merkleroot")]
	#[serde(serialize_with = "serialize_h256")]
	#[serde(deserialize_with = "deserialize_h256")]
	pub merkle_root_hash: H256,
	pub time: u32,
	pub index: u32,
	pub primary: Option<u32>,
	#[serde(rename = "nextconsensus")]
	pub next_consensus: String,
	pub witnesses: Option<Vec<NeoWitness>>,
	#[serde(rename = "tx")]
	pub transactions: Option<Vec<Transaction>>,
	pub confirmations: u32,
	#[serde(rename = "nextblockhash")]
	#[serde(serialize_with = "serialize_h256")]
	#[serde(deserialize_with = "deserialize_h256")]
	pub next_block_hash: Option<H256>,
}
