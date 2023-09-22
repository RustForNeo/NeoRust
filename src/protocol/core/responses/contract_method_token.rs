use crate::utils::*;
use primitive_types::H160;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Hash, Clone)]
pub struct ContractMethodToken {
	#[serde(deserialize_with = "deserialize_address")]
	#[serde(serialize_with = "serialize_address")]
	hash: H160,
	method: String,
	param_count: u32,
	has_return_value: bool,
	call_flags: String,
}

impl ContractMethodToken {
	pub fn new(
		hash: H160,
		method: String,
		param_count: u32,
		has_return_value: bool,
		call_flags: String,
	) -> Self {
		Self { hash, method, param_count, has_return_value, call_flags }
	}
}
