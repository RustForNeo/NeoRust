use primitive_types::H160;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Nep17Contract {
    pub script_hash: H160,
    pub symbol: String,
    pub decimals: u8,
}

impl Nep17Contract {

    pub fn new(script_hash: H160, symbol: String, decimals: u8) -> Self {
        Self {
            script_hash,
            symbol,
            decimals,
        }
    }

}