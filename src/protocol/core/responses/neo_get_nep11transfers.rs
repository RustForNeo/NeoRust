use serde::{Serialize, Deserialize};
use crate::types::hash160::H160;
use crate::types::hash256::H256;

#[derive(Serialize, Deserialize)]
pub struct NeoGetNep11Transfers {
    pub nep11_transfers: Option<Nep11Transfers>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Nep11Transfers {
    pub sent: Vec<Nep11Transfer>,
    pub received: Vec<Nep11Transfer>,
    #[serde(rename = "address")]
    pub transfer_address: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Nep11Transfer {
    #[serde(rename = "tokenid")]
    pub token_id: String,
    pub timestamp: u64,
    #[serde(rename = "assethash")]
    pub asset_hash: H160,
    #[serde(rename = "transferaddress")]
    pub transfer_address: String,
    pub amount: u64,
    #[serde(rename = "blockindex")]
    pub block_index: u32,
    #[serde(rename = "transfernotifyindex")]
    pub transfer_notify_index: u32,
    #[serde(rename = "txhash")]
    pub tx_hash: H256,
}