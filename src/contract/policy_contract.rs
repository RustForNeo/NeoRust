use crate::{
	contract::{contract_error::ContractError, smartcontract::SmartContract},
	transaction::transaction_builder::TransactionBuilder,
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyContract {
	script_hash: H160,
}

impl PolicyContract {
	pub const NAME: &'static str = "PolicyContract";
	pub const SCRIPT_HASH: H160 = SmartContract::calc_native_contract_hash(Self::NAME).unwrap();

	pub fn new() -> Self {
		Self { script_hash: Self::SCRIPT_HASH }
	}

	// Read-only methods

	pub async fn get_fee_per_byte(&self) -> Result<i32, ContractError> {
		self.call_contract_method("getFeePerByte").await
	}

	pub async fn get_exec_fee_factor(&self) -> Result<i32, ContractError> {
		self.call_contract_method("getExecFeeFactor").await
	}

	pub async fn get_storage_price(&self) -> Result<i32, ContractError> {
		self.call_contract_method("getStoragePrice").await
	}

	pub async fn is_blocked(&self, script_hash: &H160) -> Result<bool, ContractError> {
		self.call_contract_method("isBlocked", vec![script_hash.into()]).await
	}

	// State modifying methods

	pub fn set_fee_per_byte(&self, fee: i32) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("setFeePerByte", vec![fee.into()])
	}

	pub fn set_exec_fee_factor(&self, fee: i32) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("setExecFeeFactor", vec![fee.into()])
	}

	pub fn set_storage_price(&self, price: i32) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("setStoragePrice", vec![price.into()])
	}

	pub fn block_account(&self, account: &H160) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("blockAccount", vec![account.into()])
	}

	pub fn block_account_address(
		&self,
		address: &str,
	) -> Result<TransactionBuilder, ContractError> {
		let account = H160::from_address(address)?;
		self.block_account(&account)
	}

	pub fn unblock_account(&self, account: &H160) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("unblockAccount", vec![account.into()])
	}

	pub fn unblock_account_address(
		&self,
		address: &str,
	) -> Result<TransactionBuilder, ContractError> {
		let account = H160::from_address(address)?;
		self.unblock_account(&account)
	}
}
