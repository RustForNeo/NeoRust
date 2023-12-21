use crate::core::responses::{neo_transaction_result::TransactionResult, neo_witness::NeoWitness};
use primitive_types::H256;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash, Clone)]
pub struct NeoBlock {
	pub hash: H256,
	pub size: i32,
	pub version: i32,
	pub prev_block_hash: H256,
	pub merkle_root_hash: H256,
	pub time: i32,
	pub index: i32,
	pub primary: Option<i32>,
	pub next_consensus: String,
	pub witnesses: Option<Vec<NeoWitness>>,
	pub transactions: Option<Vec<TransactionResult>>,
	pub confirmations: i32,
	pub next_block_hash: Option<H256>,
}
