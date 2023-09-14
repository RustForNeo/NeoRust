use serde::{Serialize, Deserialize};
use crate::types::H256;

#[derive(Serialize, Deserialize)]
pub struct NeoGetMemPool {
    pub mem_pool_details: Option<MemPoolDetails>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct MemPoolDetails {
    pub height: u32,
    pub verified: Vec<H256>,
    pub unverified: Vec<H256>,
}