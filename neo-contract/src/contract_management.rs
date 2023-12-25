use crate::{error::ContractError, traits::smart_contract::SmartContractTrait};
use async_trait::async_trait;
use futures::{FutureExt, TryFutureExt};
use neo_providers::{
	core::{account::AccountTrait, transaction::transaction_builder::TransactionBuilder},
	JsonRpcClient, Middleware, Provider,
};
use neo_types::{
	contract_parameter::ContractParameter,
	contract_state::{ContractIdentifiers, ContractState},
	nef_file::NefFile,
	script_hash::ScriptHash,
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractManagement<'a, P: JsonRpcClient> {
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	script_hash: ScriptHash,
	#[serde(skip)]
	provider: Option<&'a Provider<P>>,
}

impl<'a, P> ContractManagement<'a, P> {
	pub fn new(script_hash: H160, provider: Option<&'a Provider<P>>) -> Self {
		Self { script_hash, provider }
	}

	pub async fn get_minimum_deployment_fee(&self) -> Result<u64, ContractError> {
		Ok(self
			.provider
			.invoke_function(&self.script_hash, "getMinimumDeploymentFee".to_string(), (), ())
			.await
			.unwrap()
			.stack[0]
			.as_int()
			.unwrap() as u64)
	}

	pub async fn set_minimum_deployment_fee(&self, fee: u64) -> Result<u64, ContractError> {
		Ok(self
			.provider
			.invoke_function(
				&self.script_hash,
				"setMinimumDeploymentFee".to_string(),
				vec![fee.into()],
				vec![],
			)
			.await
			.unwrap()
			.stack[0]
			.as_int()
			.unwrap() as u64)
	}

	pub async fn get_contract(&self, hash: H160) -> Result<ContractState, ContractError> {
		self.provider
			.get_contract_state(hash)
			.await
			.map_err(|e| ContractError::RuntimeError(e.to_string()))
	}

	pub async fn get_contract_by_id(&self, id: u32) -> Result<ContractState, ContractError> {
		let hash = self.get_contract_hash_by_id(id).await.unwrap();
		self.get_contract(hash).await
	}

	pub async fn get_contract_hash_by_id(&self, id: u32) -> Result<ScriptHash, ContractError> {
		let result = self
			.provider
			.invoke_function(&self.script_hash, "getContractById".to_string(), vec![id.into()], ())
			.await
			.unwrap()
			.stack;

		let item = &result[0];
		Ok(ScriptHash::from_slice(&item.as_bytes().unwrap()))
	}

	pub async fn get_contract_hashes(&self) -> Result<ContractIdentifiers, ContractError> {
		self.provider
			.invoke_function(&self.script_hash, "getContractHashes".to_string(), (), ())
			.await
			.map(|item| ContractIdentifiers::try_from(item).unwrap())
	}

	pub async fn has_method(
		&self,
		hash: H160,
		method: &str,
		params: usize,
	) -> Result<bool, ContractError> {
		self.provider
			.invoke_function(
				&self.script_hash,
				"hasMethod".to_string(),
				vec![hash.into(), method.into(), params.into()],
				(),
			)
			.await
			.map(|item| item.stack[0].as_bool().unwrap())
			.map_err(|e| ContractError::RuntimeError(e.to_string()))
	}

	pub async fn deploy<T: AccountTrait>(
		&self,
		nef: &NefFile,
		manifest: &[u8],
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<T, P>, ContractError> {
		let params = vec![nef.into(), manifest.into(), data.unwrap()];
		let tx = self.invoke_function("deploy", params).await;
		tx
	}
}

// Other types and helpers
#[async_trait]
impl<'a, P: JsonRpcClient> SmartContractTrait<'a, P> for ContractManagement<'a, P> {
	fn script_hash(&self) -> H160 {
		self.script_hash.clone()
	}

	fn set_script_hash(&mut self, script_hash: H160) {
		self.script_hash = script_hash;
	}

	fn provider(&self) -> Option<&Provider<P>> {
		self.provider
	}
}
