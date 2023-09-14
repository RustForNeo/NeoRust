use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct NeoValidateAddress {
    pub validation: Option<Result>,
}

#[derive(Serialize, Deserialize)]
pub struct Result {
    pub address: String,
    pub is_valid: bool,
}

impl Result {
    pub fn new(address: String, is_valid: bool) -> Self {
        Self {
            address,
            is_valid,
        }
    }
}