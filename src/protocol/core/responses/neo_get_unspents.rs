use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NeoGetUnspents {
	pub unspents: Option<Unspents>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Unspents {
	pub address: String,
	#[serde(rename = "balance")]
	pub balances: Vec<Balance>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Balance {
	#[serde(rename = "unspent")]
	pub unspent_transactions: Vec<UnspentTransaction>,
	#[serde(rename = "assethash")]
	pub asset_hash: String,
	#[serde(rename = "asset")]
	pub asset_name: String,
	#[serde(rename = "asset_symbol")]
	pub asset_symbol: String,
	pub amount: f64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct UnspentTransaction {
	#[serde(rename = "txid")]
	pub tx_id: String,
	#[serde(rename = "n")]
	pub index: u32,
	pub value: f64,
}
