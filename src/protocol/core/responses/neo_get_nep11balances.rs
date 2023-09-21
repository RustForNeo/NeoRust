use crate::utils::*;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct NeoGetNep11Balances {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub balances: Option<Nep11Balances>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Nep11Balances {
	pub address: String,
	#[serde(rename = "balance")]
	pub balances: Vec<Nep11Balance>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Nep11Balance {
	pub name: String,
	pub symbol: String,
	pub decimals: String,
	pub tokens: Vec<Nep11Token>,
	#[serde(rename = "assethash")]
	#[serde(deserialize_with = "deserialize_address")]
	#[serde(serialize_with = "serialize_address")]
	pub asset_hash: H160,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Nep11Token {
	#[serde(rename = "tokenid")]
	pub token_id: String,
	pub amount: String,
	#[serde(rename = "lastupdatedblock")]
	pub last_updated_block: u32,
}
