use crate::utils::*;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Hash)]
pub struct Diagnostics {
	pub invoked_contracts: InvokedContract,
	pub storage_changes: Vec<StorageChange>,
}

impl Diagnostics {
	pub fn new(invoked_contracts: InvokedContract, storage_changes: Vec<StorageChange>) -> Self {
		Self { invoked_contracts, storage_changes }
	}
}

#[derive(Serialize, Deserialize, Hash)]
pub struct InvokedContract {
	#[serde(deserialize_with = "deserialize_address")]
	#[serde(serialize_with = "serialize_address")]
	pub hash: H160,
	pub invoked_contracts: Option<Vec<InvokedContract>>,
}

#[derive(Serialize, Deserialize, Hash)]
pub struct StorageChange {
	pub state: String,
	pub key: String,
	pub value: String,
}
