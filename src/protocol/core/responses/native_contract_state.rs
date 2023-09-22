use crate::protocol::core::responses::{
	contract_manifest::ContractManifest, contract_nef::ContractNef,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NativeContractState {
	pub id: i32,
	pub nef: ContractNef,
	pub update_history: Vec<i32>,
	#[serde(flatten)]
	pub base: ExpressContractState,
}

impl NativeContractState {
	pub fn new(
		id: i32,
		hash: [u8; 20],
		nef: ContractNef,
		manifest: ContractManifest,
		update_history: Vec<i32>,
	) -> Self {
		Self { id, nef, update_history, base: ExpressContractState { hash, manifest } }
	}
}

#[derive(Serialize, Deserialize)]
struct ExpressContractState {
	hash: [u8; 20],
	manifest: ContractManifest,
}
