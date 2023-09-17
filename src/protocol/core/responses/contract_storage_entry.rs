use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash)]
pub struct ContractStorageEntry {
	pub key: String,
	pub value: String,
}

impl ContractStorageEntry {
	pub fn new(key: String, value: String) -> Self {
		Self { key, value }
	}
}
