use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct NeoGetNextBlockValidators {
    pub next_block_validators: Option<Vec<Validator>>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Validator {
    #[serde(rename = "publickey")]
    pub public_key: String,
    pub votes: String,
    pub active: bool,
}