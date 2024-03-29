use crate::{
	error::ContractError,
	traits::{
		fungible_token::FungibleTokenTrait, smart_contract::SmartContractTrait, token::TokenTrait,
	},
};
use async_trait::async_trait;
use neo_providers::{JsonRpcClient, Provider};
use neo_types::nns_name::NNSName;
use primitive_types::H160;

#[derive(Debug)]
pub struct FungibleTokenContract<'a, P: JsonRpcClient> {
	script_hash: H160,
	total_supply: Option<u64>,
	decimals: Option<u8>,
	symbol: Option<String>,
	provider: Option<&'a Provider<P>>,
}

impl<'a, P: JsonRpcClient> FungibleTokenContract<'a, P> {
	pub fn new(script_hash: &H160, provider: Option<&'a Provider<P>>) -> Self {
		Self {
			script_hash: script_hash.clone(),
			total_supply: None,
			decimals: None,
			symbol: None,
			provider,
		}
	}
}

#[async_trait]
impl<'a, P: JsonRpcClient> TokenTrait<'a, P> for FungibleTokenContract<'a, P> {
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

	async fn resolve_nns_text_record(&self, name: &NNSName) -> Result<H160, ContractError> {
		todo!()
	}
}

#[async_trait]
impl<'a, P: JsonRpcClient> SmartContractTrait<'a> for FungibleTokenContract<'a, P> {
	type P = P;

	fn script_hash(&self) -> H160 {
		self.script_hash
	}

	fn set_script_hash(&mut self, script_hash: H160) {
		self.script_hash = script_hash;
	}

	fn provider(&self) -> Option<&Provider<P>> {
		self.provider
	}
}

#[async_trait]
impl<'a, P: JsonRpcClient> FungibleTokenTrait<'a, P> for FungibleTokenContract<'a, P> {}
