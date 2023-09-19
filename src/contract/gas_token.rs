use crate::contract::{
	contract_error::ContractError,
	traits::{
		fungible_token::FungibleTokenTrait, smartcontract::SmartContractTrait, token::TokenTrait,
	},
};
use async_trait::async_trait;
use primitive_types::H160;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GasToken {
	script_hash: H160,
	total_supply: Option<u64>,
	decimals: Option<u8>,
	symbol: Option<String>,
}

impl GasToken {
	pub const NAME: &'static str = "GasToken";
	pub const SCRIPT_HASH: H160 = Self::calc_native_contract_hash(Self::NAME).unwrap();
	pub const DECIMALS: u8 = 8;
	pub const SYMBOL: &'static str = "GAS";

	pub fn new() -> Self {
		Self {
			script_hash: Self::SCRIPT_HASH,
			total_supply: None,
			decimals: Some(Self::DECIMALS),
			symbol: Some(Self::SYMBOL.to_string()),
		}
	}
}

#[async_trait]
impl<T> TokenTrait<T> for GasToken {
	fn total_supply(&self) -> Option<u64> {
		self.total_supply
	}

	fn set_total_supply(&mut self, total_supply: u64) {
		self.total_supply = Option::from(total_supply);
	}

	fn decimals(&self) -> Option<u8> {
		self.decimals
	}

	fn set_decimals(&mut self, decimals: u8) {
		self.decimals = Option::from(decimals);
	}

	fn symbol(&self) -> Option<String> {
		self.symbol.clone()
	}

	fn set_symbol(&mut self, symbol: String) {
		self.symbol = Option::from(symbol);
	}
}

#[async_trait]
impl<T> SmartContractTrait<T> for GasToken {
	fn script_hash(&self) -> H160 {
		self.script_hash
	}

	fn set_script_hash(&mut self, script_hash: H160) {
		self.script_hash = script_hash;
	}
}

#[async_trait]
impl<T> FungibleTokenTrait<T> for GasToken {}
