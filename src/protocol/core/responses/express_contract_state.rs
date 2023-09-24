use crate::{protocol::core::responses::contract_manifest::ContractManifest, utils::*};
use primitive_types::H160;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash, Debug, Clone)]
pub struct ExpressContractState {
	#[serde(serialize_with = "serialize_address")]
	#[serde(deserialize_with = "deserialize_address")]
	pub hash: H160,
	pub manifest: ContractManifest,
}

impl ExpressContractState {
	pub fn new(hash: H160, manifest: ContractManifest) -> Self {
		Self { hash, manifest }
	}
}

impl PartialEq for ExpressContractState {
	fn eq(&self, other: &Self) -> bool {
		self.hash == other.hash && self.manifest == other.manifest
	}
}
