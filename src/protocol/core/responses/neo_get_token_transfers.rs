use primitive_types::{H160, H256};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Serialize, Deserialize)]
pub struct NeoGetTokenTransfers<T> {
	pub transfers: Option<T>,
}

pub trait TokenTransfers: Serialize + Deserialize + Clone + PartialEq + Eq + Hash {
	type Transfer: TokenTransfer;

	fn sent(&self) -> &Vec<Self::Transfer>;
	fn received(&self) -> &Vec<Self::Transfer>;
	fn transfer_address(&self) -> &String;
}

pub trait TokenTransfer: Serialize + Deserialize + Clone + PartialEq + Eq + Hash {
	fn timestamp(&self) -> u64;
	fn asset_hash(&self) -> H160;
	fn transfer_address(&self) -> &String;
	fn amount(&self) -> u64;
	fn block_index(&self) -> u32;
	fn transfer_notify_index(&self) -> u32;
	fn tx_hash(&self) -> H256;
}
