use crate::utils::*;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct TransactionSendToken {
	#[serde(rename = "asset")]
	#[serde(deserialize_with = "deserialize_address")]
	#[serde(serialize_with = "serialize_address")]
	pub token: H160,
	pub value: i32,
	#[serde(deserialize_with = "deserialize_address")]
	#[serde(serialize_with = "serialize_address")]
	pub address: H160,
}

impl TransactionSendToken {
	pub fn new(token: H160, value: i32, address: H160) -> Self {
		Self { token, value, address }
	}
}
