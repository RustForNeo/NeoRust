use serde::{Serialize, Deserialize};
use crate::types::hash256::H256;

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