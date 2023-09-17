use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Serialize, Deserialize)]
pub struct NeoGetTokenBalances<T> {
	pub balances: Option<T>,
}

pub trait TokenBalances: Serialize + Deserialize + Clone + PartialEq + Eq + Hash {
	type Balance: TokenBalance;
	fn address(&self) -> String;
	fn balances(&self) -> &Vec<Self::Balance>;
}

pub trait TokenBalance: Serialize + Deserialize + Clone + PartialEq + Eq + Hash {
	fn asset_hash(&self) -> H160;
}
