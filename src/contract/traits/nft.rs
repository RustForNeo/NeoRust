use crate::contract::contract_error::ContractError;
use crate::contract::nns_name::NNSName;
use crate::contract::traits::token::TokenTrait;
use crate::protocol::core::stack_item::StackItem;
use crate::transaction::account_signer::AccountSigner;
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::types::contract_parameter::ContractParameter;
use crate::types::{Bytes, H160Externsion};
use crate::wallet::account::Account;
use primitive_types::H160;
use std::collections::HashMap;
use std::str::FromStr;

trait NonFungibleTokenTrait<T>: TokenTrait<T> {
	const OWNER_OF: &'static str = "ownerOf";
	const TOKENS_OF: &'static str = "tokensOf";
	const BALANCE_OF: &'static str = "balanceOf";
	const TRANSFER: &'static str = "transfer";
	const TOKENS: &'static str = "tokens";
	const PROPERTIES: &'static str = "properties";

	// Token methods

	async fn balance_of(&mut self, owner: H160) -> Result<i32, ContractError> {
		self.call_function_returning_int(NonFungibleTokenTrait::BALANCE_OF, vec![owner.into()])
	}

	// NFT methods

	async fn tokens_of(
		&mut self,
		owner: H160,
	) -> Result<dyn Iterator<Item = Bytes>, ContractError> {
		self.call_function_returning_iterator(
			NonFungibleTokenTrait::TOKENS_OF,
			vec![owner.into()],
			|item| item.try_into(),
		)
	}

	// Non-divisible NFT methods

	async fn transfer(
		&mut self,
		from: &Account,
		to: H160,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<T>, ContractError> {
		self.transfer_inner(to, token_id, data)
			.signers(vec![AccountSigner::called_by_entry(from)])
	}

	async fn transfer_inner(
		&mut self,
		to: H160,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<T>, ContractError> {
		self.throw_if_divisible_nft().await?;

		self.invoke_function(
			NonFungibleTokenTrait::TRANSFER,
			vec![to.into(), token_id.into(), data],
		)
	}

	async fn transfer_from_name(
		&mut self,
		from: &Account,
		to: &str,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<T>, ContractError> {
		self.throw_if_sender_is_not_owner(from.script_hash(), &token_id).await?;

		self.transfer_inner(H160::from_str(to)?, token_id, data)
			.signers(vec![AccountSigner::called_by_entry(from)])
	}

	async fn transfer_to_name(
		&mut self,
		to: &str,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<T>, ContractError> {
		self.throw_if_divisible_nft().await?;

		self.transfer_inner(self.resolve_nns_text_record(&NNSName::new(to)?)?, token_id, data)
	}

	async fn build_non_divisible_transfer_script(
		&mut self,
		to: H160,
		token_id: Bytes,
		data: ContractParameter,
	) -> Result<Bytes, ContractError> {
		self.throw_if_divisible_nft().await?;

		self.build_invoke_function_script(
			NonFungibleTokenTrait::TRANSFER,
			vec![Some(to.into()), Some(token_id.into()), Some(data)],
		)
	}

	async fn owner_of(&mut self, token_id: Bytes) -> Result<H160, ContractError> {
		self.throw_if_divisible_nft().await?;

		self.call_function_returning_script_hash(
			NonFungibleTokenTrait::OWNER_OF,
			vec![token_id.into()],
		)
	}

	async fn throw_if_divisible_nft(&mut self) -> Result<(), ContractError> {
		if self.get_decimals().await? != 0 {
			return Err(ContractError::InvalidStateError(
				"This method is only intended for non-divisible NFTs.".to_string(),
			));
		}

		Ok(())
	}

	async fn throw_if_sender_is_not_owner(
		&mut self,
		from: H160,
		token_id: &Bytes,
	) -> Result<(), ContractError> {
		let token_owner = self.owner_of(token_id.clone()).await?;
		if token_owner != from {
			return Err(ContractError::InvalidArgError(
				"The provided from account is not the owner of this token.".to_string(),
			));
		}

		Ok(())
	}

	// Divisible NFT methods

	async fn transfer_divisible(
		&mut self,
		from: &Account,
		to: H160,
		amount: i32,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<T>, ContractError> {
		self.transfer_divisible_from_hashes(from.script_hash(), to, amount, token_id, data)
			.signers(vec![AccountSigner::called_by_entry(from)])
	}

	async fn transfer_divisible_from_hashes(
		&mut self,
		from: H160,
		to: H160,
		amount: i32,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<T>, ContractError> {
		self.throw_if_non_divisible_nft().await?;

		self.invoke_function(
			NonFungibleTokenTrait::TRANSFER,
			vec![from.into(), to.into(), amount.into(), token_id.into(), data],
		)
	}

	async fn transfer_divisible_from_name(
		&mut self,
		from: &Account,
		to: &str,
		amount: i32,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<T>, ContractError> {
		self.transfer_divisible_from_hashes(
			from.script_hash(),
			self.resolve_nns_text_record(&NNSName::new(to)?)?,
			amount,
			token_id,
			data,
		)
		.signers(vec![AccountSigner::called_by_entry(from)])
	}

	async fn transfer_divisible_to_name(
		&mut self,
		from: H160,
		to: &str,
		amount: i32,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<T>, ContractError> {
		self.throw_if_non_divisible_nft().await?;

		self.transfer_divisible_from_hashes(
			from,
			self.resolve_nns_text_record(&NNSName::new(to)?)?,
			amount,
			token_id,
			data,
		)
	}

	fn build_divisible_transfer_script(
		&self,
		from: H160,
		to: H160,
		amount: i32,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<Bytes, ContractError> {
		self.build_invoke_function_script(
			NonFungibleTokenTrait::TRANSFER,
			vec![from.into(), to.into(), amount.into(), token_id.into(), data],
		)
	}

	async fn owners_of(
		&mut self,
		token_id: Bytes,
	) -> Result<dyn Iterator<Item = H160>, ContractError> {
		self.throw_if_non_divisible_nft().await?;

		self.call_function_returning_iterator(
			NonFungibleTokenTrait::OWNER_OF,
			vec![token_id.into()],
			|item| H160::from_address(item.address().unwrap()),
		)
	}

	async fn throw_if_non_divisible_nft(&mut self) -> Result<(), ContractError> {
		if self.get_decimals().await? == 0 {
			return Err(ContractError::InvalidStateError(
				"This method is only intended for divisible NFTs.".to_string(),
			));
		}

		Ok(())
	}

	async fn balance_of_divisible(
		&mut self,
		owner: H160,
		token_id: Bytes,
	) -> Result<i32, ContractError> {
		self.throw_if_non_divisible_nft().await?;

		self.call_function_returning_int(
			NonFungibleTokenTrait::BALANCE_OF,
			vec![owner.into(), token_id.into()],
		)
	}

	// Optional methods

	async fn tokens(&mut self) -> Result<dyn Iterator<Item = Bytes>, ContractError> {
		self.call_function_returning_iterator(NonFungibleTokenTrait::TOKENS, vec![], |item| {
			item.try_into()
		})
	}

	async fn properties(
		&mut self,
		token_id: Bytes,
	) -> Result<HashMap<String, String>, ContractError> {
		let invocation_result = self
			.call_invoke_function(NonFungibleTokenTrait::PROPERTIES, vec![token_id.into()], vec![])?
			.into_result();

		let stack_item = invocation_result.get_first_stack_item()?;
		let map = stack_item.as_map().ok_or(ContractError::UnexpectedReturnType(
			stack_item.to_json(),
			Some(vec![StackItem::MAP_VALUE.to_string()]),
		))?;

		map.iter()
			.map(|(k, v)| {
				let key = k.as_string().unwrap();
				let value = v.as_string().unwrap();
				Ok((key, value))
			})
			.collect()
	}

	async fn custom_properties(
		&mut self,
		token_id: Bytes,
	) -> Result<HashMap<String, StackItem>, ContractError> {
		let invocation_result = self
			.call_invoke_function(NonFungibleTokenTrait::PROPERTIES, vec![token_id.into()], vec![])?
			.into_result();

		let stack_item = invocation_result.get_first_stack_item()?;
		let map = stack_item.as_map().ok_or(ContractError::UnexpectedReturnType(
			stack_item.to_json(),
			Some(vec![StackItem::MAP_VALUE.to_string()]),
		))?;

		map.into_iter()
			.map(|(k, v)| {
				let key = k.as_string().unwrap();
				Ok((key, v.clone()))
			})
			.collect()
	}
}
