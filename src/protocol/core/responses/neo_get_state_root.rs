use primitive_types::H256;
use serde::{Serialize, Deserialize};
use crate::protocol::core::responses::neo_witness::NeoWitness;

#[derive(Serialize, Deserialize)]
pub struct NeoGetStateRoot {
    pub state_root: Option<StateRoot>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct StateRoot {
    pub version: u32,
    pub index: u32,
    #[serde(rename = "roothash")]
    pub root_hash: H256,
    pub witnesses: Vec<NeoWitness>,
}