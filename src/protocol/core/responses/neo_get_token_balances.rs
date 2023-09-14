use std::hash::Hash;
use serde::{Serialize, Deserialize};
use crate::types::hash160::H160;

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