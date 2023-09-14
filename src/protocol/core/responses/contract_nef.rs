use serde::{Deserialize, Serialize};
use crate::protocol::core::responses::contract_method_token::ContractMethodToken;

#[derive(Serialize, Deserialize, Hash)]
pub struct ContractNef {
    pub magic: i32,
    pub compiler: String,
    pub source: Option<String>,
    #[serde(with = "serde_with::rust::single_or_nil_array")]
    pub tokens: Vec<ContractMethodToken>,
    pub script: String,
    pub checksum: i32
}

impl ContractNef {
    pub fn new(magic: i32, compiler: String, source: Option<String>, tokens: Vec<ContractMethodToken>, script: String, checksum: i32) -> Self {
        Self {
            magic,
            compiler,
            source,
            tokens,
            script,
            checksum
        }
    }
}