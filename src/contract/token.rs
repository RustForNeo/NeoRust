use crate::protocol::core::record_type::RecordType;
use crate::types::H160Externsion;
use crate::{
	contract::{contract_error::ContractError, name_service, nns_name::NNSName},
	protocol::neo_rust::NeoRust,
};
use decimal::d128;
use primitive_types::H160;
use serde::{Deserialize, Serialize};

// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub struct Token {
// 	script_hash: H160,
// 	total_supply: Option<u64>,
// 	decimals: Option<u8>,
// 	symbol: Option<String>,
// }

pub trait Token {
	const TOTAL_SUPPLY: &'static str = "totalSupply";
	const SYMBOL: &'static str = "symbol";
	const DECIMALS: &'static str = "decimals";

	fn script_hash(&self) -> H160;
	fn set_script_hash(&self, script_hash: H160);
	fn total_supply(&self) -> Option<u64>;

	fn set_total_supply(&self, total_supply: u64);

	fn decimals(&self) -> Option<u8>;

	fn set_decimals(&self, decimals: u8);

	fn symbol(&self) -> Option<String>;

	fn set_symbol(&self, symbol: String);

	// fn new(script_hash: H160) -> Self {
	// 	Self { script_hash, total_supply: None, decimals: None, symbol: None }
	// }

	async fn get_total_supply(&mut self) -> Result<u64, ContractError> {
		if let Some(supply) = &self.total_supply() {
			return Ok(supply.clone().into());
		}

		let supply = self.call_function_returning_int(Self::TOTAL_SUPPLY, vec![]).await? as u64;

		self.set_total_supply(supply);
		Ok(supply)
	}

	async fn get_decimals(&mut self) -> Result<u8, ContractError> {
		if let Some(decimals) = &self.decimals() {
			return Ok(decimals.clone().into());
		}

		let decimals = self.call_function_returning_int(Self::DECIMALS, vec![]).await? as u8;

		self.set_decimals(decimals);
		Ok(decimals)
	}

	// Other methods

	async fn get_symbol(&mut self) -> Result<String, ContractError> {
		if let Some(symbol) = &self.symbol() {
			return Ok(symbol.clone());
		}

		let symbol = self.call_function_returning_string(Self::SYMBOL, vec![]).await?;

		self.set_symbol(symbol.clone());
		Ok(symbol)
	}

	async fn to_fractions(&mut self, amount: d128) -> Result<u64, ContractError> {
		let a = d128!(1.1);
		let decimals = self.get_decimals().await?;
		Self::to_fractions_decimal(amount, decimals)
	}

	fn to_fractions_decimal(amount: d128, decimals: u8) -> Result<u64, ContractError> {
		if amount.scale() > decimals {
			return Err(ContractError::InvalidArgError("Too many decimal places".to_string()));
		}

		let scaled = d128::from(10u64.pow(decimals.into())) * amount;
		Ok(scaled.as_u64().unwrap())
	}

	// Other helper methods
	async fn to_decimals_u64(&mut self, amount: u64) -> Result<d128, ContractError> {
		let decimals = self.get_decimals().await?;
		Ok(Self::to_decimals(amount, decimals))
	}

	fn to_decimals(amount: u64, decimals: u8) -> d128 {
		let mut dec = d128::from(amount);
		if decimals > 0 {
			dec /= d128::from(10_u64.pow(decimals.into()));
		} else if decimals < 0 {
			dec *= d128::from(10_u64.pow(-decimals.into()));
		}
		dec
	}

	async fn resolve_nns_text_record(&self, name: &NNSName) -> Result<H160, ContractError> {
		let address = NeoRust::instance()
			.as_ref()
			.unwrap()
			.call_contract_func(
				name_service::NeoNameService::CONTRACT_HASH,
				"resolve",
				vec![name.to_param()?, RecordType::TXT.to_param()?],
			)
			.await?
			.pop()
			.and_then(|item| item.as_address())
			.map(H160::from_address)
			.ok_or_else(|| ContractError::RuntimeError("Address{&name}".to_string()))?;

		Ok(address)
	}
}
