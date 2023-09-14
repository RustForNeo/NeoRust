use primitive_types::H160;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct NeoGetNep17Balances {
    pub balances: Option<Nep17Balances>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Nep17Balances {
    pub address: String,
    #[serde(rename = "balance")]
    pub balances: Vec<Nep17Balance>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Nep17Balance {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<String>,
    pub amount: String,
    #[serde(rename = "lastupdatedblock")]
    pub last_updated_block: f64,
    #[serde(rename = "assethash")]
    pub asset_hash: H160,
}
