use crate::utils::*;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct Nep17Contract {
	#[serde(serialize_with = "serialize_address")]
	#[serde(deserialize_with = "deserialize_address")]
	pub script_hash: H160,
	pub symbol: String,
	pub decimals: u8,
}

impl Nep17Contract {
	pub fn new(script_hash: H160, symbol: String, decimals: u8) -> Self {
		Self { script_hash, symbol, decimals }
	}
}
