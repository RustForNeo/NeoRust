use crate::{
	contract::{
		contract_error::ContractError, fungible_token::FungibleToken, gas_token::GasToken,
		neo_token::NeoToken,
	},
	transaction::transaction_builder::TransactionBuilder,
	types::{contract_parameter::ContractParameterType::String, H160Externsion},
	wallet::account::Account,
};
use decimal::d128;
use primitive_types::H160;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeoURI {
	uri: Option<Url>,
	recipient: Option<H160>,
	token: Option<H160>,
	amount: Option<d128>,
}

impl NeoURI {
	const NEO_SCHEME: &'static str = "neo";
	const MIN_NEP9_URI_LENGTH: usize = 38;
	const NEO_TOKEN_STRING: &'static str = "neo";
	const GAS_TOKEN_STRING: &'static str = "gas";

	pub fn new() -> Self {
		Self { uri: None, recipient: None, token: None, amount: None }
	}

	pub fn from_uri(uri_string: &str) -> Result<Self, ContractError> {
		let parts: Vec<&str> = uri_string.split("?").collect();
		let base = parts[0];
		let query = if parts.len() > 1 { Some(parts[1]) } else { None };

		let base_parts: Vec<&str> = base.split(":").collect();
		if base_parts.len() != 2
			|| base_parts[0] != Self::NEO_SCHEME
			|| uri_string.len() < Self::MIN_NEP9_URI_LENGTH
		{
			return Err(ContractError::InvalidNeoName("Invalid NEP-9 URI".to_string()));
		}

		let mut neo_uri = Self::new().to(H160::from_address(base_parts[1])?);

		if let Some(query_str) = query {
			for part in query_str.split("&") {
				let kv: Vec<&str> = part.split("=").collect();
				if kv.len() != 2 {
					return Err(ContractError::InvalidNeoName("Invalid query".to_string()));
				}

				match kv[0] {
					"asset" if neo_uri.token.is_none() => {
						neo_uri.token(kv[1].to_owned())?;
					},
					"amount" if neo_uri.amount.is_none() => {
						neo_uri.amount = Some(kv[1].parse()?);
					},
					_ => {},
				}
			}
		}

		Ok(neo_uri)
	}

	// Getters

	pub fn uri_string(&self) -> Option<String> {
		self.uri.as_ref().map(|uri| uri.to_string())
	}

	pub fn recipient_address(&self) -> Option<String> {
		self.recipient.as_ref().map(H160::to_address)
	}

	pub fn token_string(&self) -> Option<String> {
		self.token.as_ref().map(|token| match token {
			token if *token == NeoToken::SCRIPT_HASH => Self::NEO_TOKEN_STRING.to_owned(),
			token if *token == GasToken::SCRIPT_HASH => Self::GAS_TOKEN_STRING.to_owned(),
			_ => token.to_string(),
		})
	}

	// Builders

	pub async fn build_transfer_from(
		&self,
		sender: &Account,
	) -> Result<TransactionBuilder, dyn Error> {
		let recipient = self
			.recipient
			.ok_or(ContractError::InvalidStateError("Recipient not set".to_string()))?;
		let amount = self
			.amount
			.ok_or(ContractError::InvalidStateError("Amount not set".to_string()))?;
		let token = self
			.token
			.ok_or(ContractError::InvalidStateError("Token not set".to_string()))?;

		let token = FungibleToken::new(token);

		// Validate amount precision

		let amount_scale = amount.scale();

		if Self::is_neo_token(&token) && amount_scale > 0 {
			return Err(ContractError::InvalidArgError("NEO does not support decimals".to_string()));
		}

		if Self::is_gas_token(&token) && amount_scale > GasToken::decimals() {
			return Err(ContractError::InvalidArgError(
				"Too many decimal places for GAS".to_string(),
			));
		}

		let decimals = token.get_decimals().await?;
		if amount_scale > decimals {
			return Err(ContractError::InvalidArgError(
				"Too many decimal places for token".to_string(),
			));
		}

		token.transfer(sender, recipient, token.to_fractions(amount)).await
	}

	// Helpers

	fn is_neo_token(token: &H160) -> bool {
		token == &NeoToken::script_hash()
	}

	fn is_gas_token(token: &H160) -> bool {
		token == &GasToken::script_hash()
	}

	// Setters

	pub fn to(mut self, recipient: H160) -> Self {
		self.recipient = Some(recipient);
		self
	}

	pub fn token(mut self, token: H160) -> Self {
		self.token = Some(token);
		self
	}

	pub fn token_str(mut self, token_str: &str) -> Result<Self, dyn Error> {
		self.token = match token_str {
			Self::NEO_TOKEN_STRING => Some(NeoToken::script_hash()),
			Self::GAS_TOKEN_STRING => Some(GasToken::script_hash()),
			_ => Some(token_str.parse()?),
		};
		Ok(self)
	}

	pub fn amount(mut self, amount: d128) -> Self {
		self.amount = Some(amount);
		self
	}

	// URI builder

	fn build_query(&self) -> String {
		let mut parts = Vec::new();

		if let Some(token) = &self.token {
			let token_str = match token {
				token if *token == NeoToken::script_hash() => Self::NEO_TOKEN_STRING.to_owned(),
				token if *token == GasToken::script_hash() => Self::GAS_TOKEN_STRING.to_owned(),
				_ => token.to_string(),
			};

			parts.push(format!("asset={}", token_str));
		}

		if let Some(amount) = &self.amount {
			parts.push(format!("amount={}", amount));
		}

		parts.join("&")
	}

	pub fn build_uri(mut self) -> Result<Self, dyn Error> {
		let recipient = self
			.recipient
			.ok_or(ContractError::InvalidStateError("No recipient set".to_string()))?;

		let base = format!("{}:{}", Self::NEO_SCHEME, recipient.to_address()?);
		let query = self.build_query();
		let uri_str = if query.is_empty() { base } else { format!("{}?{}", base, query) };

		self.uri = Some(uri_str.parse()?);
		Ok(self)
	}
}
