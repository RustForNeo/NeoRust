use primitive_types::H256;
use serde::{Serialize, Deserialize};

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