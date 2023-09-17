use crate::contract::contract_error::ContractError;
use crate::contract::nns_name::NNSName;
use crate::contract::token::Token;
use crate::protocol::core::neo_trait::NeoTrait;
use crate::protocol::core::responses::invocation_result::InvocationResult;
use crate::protocol::core::stack_item::StackItem;
use crate::protocol::neo_rust::NeoRust;
use crate::transaction::signer::Signer;
use crate::types::Bytes;
use crate::{
	transaction::transaction_builder::TransactionBuilder,
	types::contract_parameter::ContractParameter, wallet::account::Account,
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonFungibleToken {
	script_hash: H160,
	total_supply: Option<u64>,
	symbol: Option<String>,
	decimals: Option<u8>,
}
impl<T> NonFungibleToken
where
	T: Signer,
{
	// Constants
	const OWNER_OF: &'static str = "ownerOf";
	const TRANSFER: &'static str = "transfer";

	// Methods
	fn balance_of(&self, owner: H160) -> Result<i32, ContractError> {
		let result = self.call_function("balanceOf", [owner.into()])?;

		Ok(result.as_int()?)
	}

	fn tokens_of(&self, owner: H160) -> Result<dyn Iterator<Item = Bytes>, ContractError> {
		let results = self.call_function("tokensOf", [owner.into()])?;

		let tokens = results.as_array()?.iter().map(|item| Bytes::from(item.as_bytes()?)).collect();

		Ok(tokens.into_iter())
	}

	fn transfer(
		&mut self,
		from: Account,
		to: H160,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<T>, ContractError> {
		let params = [to.into(), token_id.into(), data.unwrap_or_default().into()];

		let tx_builder = self.invoke_function("transfer", params)?;

		Ok(tx_builder.signers(from))
	}

	fn transfer_from_account(
		&mut self,
		from: Account,
		to: H160,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<T>, ContractError> {
		let tx_builder = self.transfer(from, to, token_id, data)?;
		Ok(tx_builder)
	}

	fn transfer_to_name(
		&mut self,
		to: NNSName,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<T>, ContractError> {
		let to_address = self.resolve_nns_name(&to)?;
		let tx_builder = self.transfer(from, to_address, token_id, data)?;
		Ok(tx_builder)
	}

	fn build_transfer_script(
		&self,
		to: H160,
		token_id: Bytes,
		data: ContractParameter,
	) -> Result<Bytes, ContractError> {
		let script = self
			.build_invoke_function_script("transfer", [to.into(), token_id.into(), data.into()])?;

		Ok(script)
	}

	fn owner_of(&self, token_id: Bytes) -> Result<H160, ContractError> {
		let owner = self.call_function("ownerOf", [token_id.into()])?.as_address()?;

		Ok(H160::from(owner))
	}

	async fn throw_if_divisible(&mut self) -> Result<(), ContractError> {
		let decimals = self.get_decimals().await?;
		if decimals != 0 {
			return Err(ContractError::InvalidStateError(
				"This method is only for non-divisible NFTs".to_string(),
			));
		}
		Ok(())
	}

	fn throw_if_not_owner(&self, from: H160, token_id: Bytes) -> Result<(), ContractError> {
		let owner = self.owner_of(token_id)?;
		if from != owner {
			return Err(ContractError::InvalidArgError(
				"Provided account is not owner of token".to_string(),
			));
		}
		Ok(())
	}
	fn transfer_divisible(
		&mut self,
		from: Account,
		to: H160,
		amount: i32,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<T>, ContractError> {
		let params = [
			from.into(),
			to.into(),
			amount.into(),
			token_id.into(),
			data.unwrap_or_default().into(),
		];

		let tx = self.invoke_function("transferDivisible", params)?;
		Ok(tx)
	}

	fn transfer_divisible_from(
		&mut self,
		from: H160,
		to: H160,
		amount: i32,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<T>, ContractError> {
		let params = [
			from.into(),
			to.into(),
			amount.into(),
			token_id.into(),
			data.unwrap_or_default().into(),
		];

		let tx = self.invoke_function("transferDivisible", params)?;
		Ok(tx)
	}

	fn transfer_divisible_to_name(
		&mut self,
		from: H160,
		to: NNSName,
		amount: i32,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<T>, ContractError> {
		let to_addr = self.resolve_nns_name(&to)?;

		let params = [
			from.into(),
			to_addr.into(),
			amount.into(),
			token_id.into(),
			data.unwrap_or_default().into(),
		];

		let tx = self.invoke_function("transferDivisible", params)?;
		Ok(tx)
	}

	fn build_divisible_transfer_script(
		&self,
		from: H160,
		to: H160,
		amount: i32,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<Bytes, ContractError> {
		let params = [
			from.into(),
			to.into(),
			amount.into(),
			token_id.into(),
			data.unwrap_or_default().into(),
		];

		let script = self.build_invoke_function_script("transferDivisible", params)?;
		Ok(script)
	}

	fn owners_of(&self, token_id: Bytes) -> Result<dyn Iterator<Item = ()>, ContractError> {
		let results = self.call_function("ownersOf", [token_id.into()])?;

		let owners =
			results.as_array()?.iter().map(|item| H160::from(item.as_address()?)).collect();

		Ok(owners.into_iter())
	}

	fn throw_if_not_divisible(&mut self) -> Result<(), ContractError> {
		let decimals = self.get_decimals()?;
		if decimals == 0 {
			return Err(ContractError::InvalidStateError(
				"This method is only for divisible NFTs".to_string(),
			));
		}
		Ok(())
	}

	fn balance_of_divisible(&self, owner: H160, token_id: Bytes) -> Result<i32, ContractError> {
		let balance = self
			.call_function("balanceOfDivisible", [owner.into(), token_id.into()])?
			.as_int()?;
		Ok(balance)
	}

	fn tokens(&self) -> Result<dyn Iterator<Item = Bytes>, ContractError> {
		let results = self.call_function("tokens", [])?;
		let tokens = results.as_array()?.iter().map(|item| Bytes::from(item.as_bytes()?)).collect();
		Ok(tokens.into_iter())
	}

	fn properties(&self, token_id: Bytes) -> Result<HashMap<String, String>, ContractError> {
		let result = self.call_function("properties", [token_id.into()])?.as_map()?;

		let mut map = HashMap::new();
		for (key, value) in result {
			map.insert(key.as_string()?, value.as_string()?);
		}

		Ok(map)
	}

	fn custom_properties(
		&self,
		token_id: Bytes,
	) -> Result<HashMap<String, StackItem>, ContractError> {
		let result = NeoRust::instance().invoke_function(
			&self.script_hash,
			"customProperties".to_string(),
			vec![token_id],
		);

		let mut map = HashMap::new();
		if let StackItem::Map(items) = result {
			for item in items {
				if let StackItem::Array(key) = item.key {
					map.insert(key.to_hex(), item.value);
				}
			}
		}

		Ok(map)
	}

	fn map_stack_item(
		&self,
		invocation_result: InvocationResult,
	) -> Result<StackItem, ContractError> {
		let stack_item = invocation_result.get_first_stack_item()?;

		if let StackItem::Map = stack_item {
			Ok(stack_item.clone())
		} else {
			Err(ContractError::UnexpectedReturnType(
				stack_item.to_string(),
				Some(vec![StackItem::Map.to_string()]),
			))
		}
	}
}

impl Token for NonFungibleToken {
	fn script_hash(&self) -> H160 {
		self.script_hash
	}

	fn set_script_hash(&mut self, script_hash: H160) {
		self.script_hash = script_hash;
	}

	fn total_supply(&self) -> Option<u64> {
		self.total_supply
	}

	fn set_total_supply(&mut self, total_supply: u64) {
		self.total_supply = Some(total_supply);
	}

	fn decimals(&self) -> Option<u8> {
		self.decimals
	}

	fn set_decimals(&mut self, decimals: u8) {
		self.decimals = Some(decimals);
	}

	fn symbol(&self) -> Option<String> {
		self.symbol.clone()
	}

	fn set_symbol(&mut self, symbol: String) {
		self.symbol = Some(symbol);
	}
}
