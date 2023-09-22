// contract_management

use crate::{
	contract::{
		contract_error::ContractError, nef_file::NefFile, traits::smartcontract::SmartContractTrait,
	},
	neo_error::NeoError,
	protocol::{
		core::{
			neo_trait::NeoTrait,
			responses::contract_state::{ContractIdentifiers, ContractState},
		},
		http_service::HttpService,
		neo_rust::NeoRust,
	},
	transaction::transaction_builder::TransactionBuilder,
	types::{contract_parameter::ContractParameter, H160Externsion},
	utils::*,
};
use async_trait::async_trait;
use futures::{FutureExt, TryFutureExt};
use primitive_types::H160;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractManagement {
	#[serde(deserialize_with = "deserialize_address")]
	#[serde(serialize_with = "serialize_address")]
	script_hash: H160,
}

impl ContractManagement {
	pub fn new(script_hash: H160) -> Self {
		Self { script_hash }
	}

	pub async fn get_minimum_deployment_fee(&self) -> Result<u64, NeoError> {
		Ok(NeoRust::instance()
			.invoke_function(
				&self.script_hash,
				"getMinimumDeploymentFee".to_string(),
				vec![],
				vec![],
			)
			.request()
			.await
			.unwrap()
			.stack[0]
			.as_int()
			.unwrap() as u64)
	}

	pub async fn set_minimum_deployment_fee(&self, fee: u64) -> Result<u64, NeoError> {
		Ok(NeoRust::instance()
			.invoke_function(
				&self.script_hash,
				"setMinimumDeploymentFee".to_string(),
				vec![fee.into()],
				vec![],
			)
			.request()
			.await
			.unwrap()
			.stack[0]
			.as_int()
			.unwrap() as u64)
	}

	pub async fn get_contract(&self, hash: H160) -> Result<ContractState, ContractError> {
		NeoRust::instance()
			.get_contract_state(hash)
			.request()
			.await
			.map_err(|e| ContractError::RuntimeError(e.to_string()))
	}

	pub async fn get_contract_by_id(&self, id: u32) -> Result<ContractState, ContractError> {
		let hash = self.get_contract_hash_by_id(id).await.unwrap();
		self.get_contract(hash).await
	}

	pub async fn get_contract_hash_by_id(&self, id: u32) -> Result<H160, ContractError> {
		let result = NeoRust::instance()
			.invoke_function(
				&self.script_hash,
				"getContractById".to_string(),
				vec![id.into()],
				vec![],
			)
			.request()
			.await
			.unwrap()
			.stack;

		let item = &result[0];
		Ok(H160::from_slice(&item.as_bytes().unwrap()))
	}

	pub async fn get_contract_hashes(&self) -> Result<ContractIdentifiers, NeoError> {
		NeoRust::instance()
			.invoke_function(&self.script_hash, "getContractHashes".to_string(), vec![], vec![])
			.request()
			.await
			.map(|item| ContractIdentifiers::try_from(item).unwrap())
	}

	pub async fn has_method(
		&self,
		hash: H160,
		method: &str,
		params: usize,
	) -> Result<bool, ContractError> {
		NeoRust::instance()
			.invoke_function(
				&self.script_hash,
				"hasMethod".to_string(),
				vec![hash.into(), method.into(), params.into()],
				vec![],
			)
			.request()
			.await
			.map(|item| item.stack[0].as_bool().unwrap())
			.map_err(|e| ContractError::RuntimeError(e.to_string()))
	}

	pub async fn deploy(
		&self,
		nef: &NefFile,
		manifest: &[u8],
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder, NeoError> {
		let params = vec![nef.into(), manifest.into(), data.unwrap()];
		let tx = self.invoke_function("deploy", params).await;
		tx.map_err(|e| NeoError::ContractError(e))
	}
}

// Other types and helpers
#[async_trait]
impl SmartContractTrait for ContractManagement {
	fn script_hash(&self) -> H160 {
		self.script_hash.clone()
	}

	fn set_script_hash(&mut self, script_hash: H160) {
		self.script_hash = script_hash;
	}
}
