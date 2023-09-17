use crate::wallet::nep6contract::NEP6Contract;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct NEP6Account {
	pub address: String,
	pub label: Option<String>,
	#[serde(default)]
	pub is_default: bool,
	pub lock: bool,
	pub key: Option<String>,
	pub contract: Option<NEP6Contract>,
	pub extra: Option<HashMap<String, String>>,
}

impl NEP6Account {
	pub fn new(
		address: String,
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
