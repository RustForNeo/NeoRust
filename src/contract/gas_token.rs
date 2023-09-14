use primitive_types::H160;
use serde::{Deserialize, Serialize};
use crate::contract::contract_error::ContractError;
use crate::contract::fungible_token::FungibleToken;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GasToken {
    script_hash: H160,
}

impl GasToken {

    pub const NAME: &'static str = "GasToken";
    pub const SCRIPT_HASH: H160 = H160::ZERO;//""// compute hash
    pub const DECIMALS: u8 = 8;
    pub const SYMBOL: &'static str = "GAS";

    pub fn new() -> Self {
        Self {
            script_hash: Self::SCRIPT_HASH,
        }
    }

    fn get_name(&self) -> Result<Option<String>, ContractError> {
        Ok(Some(Self::NAME.to_string()))
    }

    fn get_symbol(&self) -> Result<String, ContractError> {
        Ok(Self::SYMBOL.to_string())
    }

    fn get_decimals(&self) -> Result<u8, ContractError> {
        Ok(Self::DECIMALS)
    }
}

impl FungibleToken for GasToken {
}