use decimal::d128;
use serde::{Deserialize, Serialize};
use crate::contract::contract_error::ContractError;
use crate::contract::name_service;
use crate::contract::name_service::RecordType;
use crate::contract::nns_name::NNSName;
use crate::types::hash160::H160;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    script_hash: H160,
    total_supply: Option<u64>,
    decimals: Option<u8>,
    symbol: Option<String>,
}

impl Token {

    const TOTAL_SUPPLY: &'static str = "totalSupply";
    const SYMBOL: &'static str = "symbol";
    const DECIMALS: &'static str = "decimals";

    pub fn new(script_hash: H160) -> Self {
        Self {
            script_hash,
            total_supply: None,
            decimals: None,
            symbol: None,
        }
    }

    pub async fn get_total_supply(&mut self) -> Result<u64, ContractError> {
        if let Some(supply) = &self.total_supply {
            return Ok(supply.clone().into());
        }

        let supply = self.call_function_returning_int(Self::TOTAL_SUPPLY, vec![])
            .await? as u64;

        self.total_supply = Some(supply);
        Ok(supply)
    }

    pub async fn get_decimals(&mut self) -> Result<u8, ContractError> {
        if let Some(decimals) = &self.decimals {
            return Ok(decimals.clone().into());
        }

        let decimals = self.call_function_returning_int(Self::DECIMALS, vec![])
            .await? as u8;

        self.decimals = Some(decimals);
        Ok(decimals)
    }

    // Other methods

    pub async fn get_symbol(&mut self) -> Result<String, ContractError> {
        if let Some(symbol) = &self.symbol {
            return Ok(symbol.clone());
        }

        let symbol = self.call_function_returning_string(Self::SYMBOL, vec![])
            .await?;

        self.symbol = Some(symbol.clone());
        Ok(symbol)
    }

    pub async fn to_fractions(&self, amount: d128) -> Result<u64, ContractError> {
        let a = d128!(1.1);
        let decimals = self.get_decimals().await?;
        Self::to_fractions(amount, decimals)
    }

    pub fn to_fractions_decimal(amount: d128, decimals: u8) -> Result<u64, ContractError> {

        if amount.scale() > decimals {
            return Err(invalid_arg_error("Too many decimal places"));
        }

        let scaled = d128::from(10u64.pow(decimals.into())) * amount;
        Ok(scaled.as_u64().unwrap())
    }

    // Other helper methods
    pub async fn to_decimals(&self, amount: u64) -> Result<d128, ContractError> {
        let decimals = self.get_decimals().await?;
        Self::to_decimals(amount, decimals)
    }

    pub fn to_decimals(amount: u64, decimals: u8) -> d128 {
        let mut dec = d128::from(amount);
        if decimals > 0 {
            dec /= d128::from(10_u64.pow(decimals.into()));
        } else if decimals < 0 {
            dec *= d128::from(10_u64.pow(-decimals.into()));
        }
        dec
    }

    async fn resolve_nns_text_record(&self, name: &NNSName) -> Result<H160, ContractError> {
        let address = NeoRust::instance().as_ref()
            .unwrap()
            .call_contract_func(
                name_service::CONTRACT_HASH,
                "resolve",
                vec![name.to_param()?, RecordType::Txt.to_param()?]
            )
            .await?
            .pop()
            .and_then(|item| item.as_address())
            .map(H160::from_address)
            .ok_or_else(|| invalid_return_type_error("Address", &name))?;

        Ok(address)
    }

}