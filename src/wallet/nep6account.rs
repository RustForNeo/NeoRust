use crate::{types::Address, utils::*, wallet::nep6contract::NEP6Contract};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NEP6Account {
	#[serde(deserialize_with = "deserialize_address")]
	#[serde(serialize_with = "serialize_address")]
	pub address: Address,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label: Option<String>,
	#[serde(default)]
	pub is_default: bool,
	pub lock: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub key: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub contract: Option<NEP6Contract>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub extra: Option<HashMap<String, String>>,
}

impl NEP6Account {
	pub fn new(
		address: Address,
		label: Option<String>,
		is_default: bool,
		lock: bool,
		key: Option<String>,
		contract: Option<NEP6Contract>,
		extra: Option<HashMap<String, String>>,
	) -> Self {
		Self { address, label, is_default, lock, key, contract, extra }
	}
}

impl PartialEq for NEP6Account {
	fn eq(&self, other: &Self) -> bool {
		self.address == other.address
	}
}
