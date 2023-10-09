use crate::{traits::smartcontract::SmartContractTrait, transaction_builder::TransactionBuilder};
use async_trait::async_trait;
use neo_types::{
	contract_error::ContractError,
	script_hash::{ScriptHash, ScriptHashExtension},
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyContract {
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	script_hash: ScriptHash,
}

impl PolicyContract {
	pub const NAME: &'static str = "PolicyContract";
	// pub const SCRIPT_HASH: H160 = Self::calc_native_contract_hash(Self::NAME).unwrap();

	pub fn new() -> Self {
		Self { script_hash: Self::calc_native_contract_hash(Self::NAME).unwrap() }
	}

	// Read-only methods

	pub async fn get_fee_per_byte(&self) -> Result<i32, ContractError> {
		self.call_function_returning_int("getFeePerByte", vec![]).await
	}

	pub async fn get_exec_fee_factor(&self) -> Result<i32, ContractError> {
		self.call_function_returning_int("getExecFeeFactor", vec![]).await
	}

	pub async fn get_storage_price(&self) -> Result<i32, ContractError> {
		self.call_function_returning_int("getStoragePrice", vec![]).await
	}

	pub async fn is_blocked(&self, script_hash: &H160) -> Result<bool, ContractError> {
		self.call_function_returning_bool("isBlocked", vec![script_hash.into()]).await
	}

	// State modifying methods

	pub async fn set_fee_per_byte(&self, fee: i32) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("setFeePerByte", vec![fee.into()]).await
	}

	pub async fn set_exec_fee_factor(&self, fee: i32) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("setExecFeeFactor", vec![fee.into()]).await
	}

	pub async fn set_storage_price(&self, price: i32) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("setStoragePrice", vec![price.into()]).await
	}

	pub async fn block_account(&self, account: &H160) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("blockAccount", vec![account.into()]).await
	}

	pub async fn block_account_address(
		&self,
		address: &str,
	) -> Result<TransactionBuilder, ContractError> {
		let account = ScriptHash::from_address(address).unwrap();
		self.block_account(&account).await
	}

	pub async fn unblock_account(
		&self,
		account: &H160,
	) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("unblockAccount", vec![account.into()]).await
	}

	pub async fn unblock_account_address(
		&self,
		address: &str,
	) -> Result<TransactionBuilder, ContractError> {
		let account = ScriptHash::from_address(address).unwrap();
		self.unblock_account(&account).await
	}
}

#[async_trait]
impl SmartContractTrait for PolicyContract {
	fn script_hash(&self) -> H160 {
		self.script_hash
	}

	fn set_script_hash(&mut self, script_hash: H160) {
		self.script_hash = script_hash;
	}
}
