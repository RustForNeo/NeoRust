use bitcoin::Transaction;
use primitive_types::H256;
use serde::{Serialize, Deserialize};
use crate::protocol::core::responses::neo_witness::NeoWitness;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct NeoBlock {
    pub hash: H256,
    pub size: u32,
    pub version: u32,
    #[serde(rename = "previousblockhash")]
    pub prev_block_hash: H256,
    #[serde(rename = "merkleroot")]
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
    pub next_block_hash: Option<H256>,
}