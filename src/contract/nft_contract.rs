use crate::contract::traits::{
	nft::NonFungibleTokenTrait, smartcontract::SmartContractTrait, token::TokenTrait,
};
use async_trait::async_trait;
use primitive_types::H160;

#[derive(Debug)]
pub struct NftContract {
	script_hash: H160,
	total_supply: Option<u64>,
	decimals: Option<u8>,
	symbol: Option<String>,
}

impl NftContract {
	pub fn new(script_hash: &H160) -> Self {
		Self { script_hash: script_hash.clone(), total_supply: None, decimals: None, symbol: None }
	}
}

#[async_trait]
impl TokenTrait for NftContract {
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
impl SmartContractTrait for NftContract {
	fn script_hash(&self) -> H160 {
		self.script_hash
	}

	fn set_script_hash(&mut self, script_hash: H160) {
		self.script_hash = script_hash;
	}
}

#[async_trait]
impl NonFungibleTokenTrait for NftContract {}