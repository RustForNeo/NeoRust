use crate::{
	contract::{
		contract_error::ContractError,
		traits::{
			fungible_token::FungibleTokenTrait, smartcontract::SmartContractTrait,
			token::TokenTrait,
		},
	},
	protocol::core::{responses::neo_account_state::AccountState, stack_item::StackItem},
	transaction::transaction_builder::TransactionBuilder,
	wallet::account::Account,
};
use async_trait::async_trait;
use p256::PublicKey;
use primitive_types::H160;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeoToken {
	script_hash: H160,
	total_supply: Option<u64>,
	decimals: Option<u8>,
	symbol: Option<String>,
}

impl<T> NeoToken {
	pub const NAME: &'static str = "NeoToken";
	pub const SCRIPT_HASH: H160 = Self::calc_native_contract_hash(Self::NAME).unwrap();
	pub const DECIMALS: u8 = 0;
	pub const SYMBOL: &'static str = "NEO";
	pub const TOTAL_SUPPLY: u64 = 100_000_000;

	fn new() -> Self {
		NeoToken {
			script_hash: Self::SCRIPT_HASH,
			total_supply: Some(Self::TOTAL_SUPPLY),
			decimals: Some(Self::DECIMALS),
			symbol: Some(Self::SYMBOL.to_string()),
		}
	}

	// Unclaimed Gas

	async fn unclaimed_gas(
		&self,
		account: &Account,
		block_height: i32,
	) -> Result<i64, ContractError> {
		self.unclaimed_gas(account, block_height).await
	}

	async fn unclaimed_gas_contract(
		&self,
		script_hash: &H160,
		block_height: i32,
	) -> Result<i64, ContractError> {
		Ok(self
			.call_function_returning_int(
				"unclaimedGas",
				vec![script_hash.into(), block_height.into()],
			)
			.await
			.unwrap() as i64)
	}

	// Candidate Registration

	fn register_candidate(
		&self,
		candidate_key: &PublicKey,
	) -> Result<TransactionBuilder<T>, ContractError> {
		self.invoke_function("registerCandidate", vec![candidate_key.to_stack_item()])
	}

	fn unregister_candidate(
		&self,
		candidate_key: &PublicKey,
	) -> Result<TransactionBuilder<T>, ContractError> {
		self.invoke_function("unregisterCandidate", vec![candidate_key.to_stack_item()])
	}

	// Committee and Candidates Information

	async fn get_committee(&self) -> Result<Vec<PublicKey>, ContractError> {
		self.call_function_returning_list_of_keys("getCommittee").await
	}

	async fn get_candidates(&self) -> Result<Vec<Candidate>, ContractError> {
		let candidates = self.call_function_returning_candidates("getCandidates").await?;
		candidates.into_iter().map(Candidate::from).collect()
	}

	async fn is_candidate(&self, public_key: &PublicKey) -> Result<bool, ContractError> {
		Ok(self.get_candidates().await?.into_iter().any(|c| c.public_key == *public_key))
	}

	// Voting

	async fn vote(
		&self,
		voter: &H160,
		candidate: Option<&PublicKey>,
	) -> Result<TransactionBuilder<T>, ContractError> {
		let params = match candidate {
			Some(key) => vec![voter.into(), key.to_stack_item()],
			None => vec![voter.into(), StackItem::null()],
		};

		self.invoke_function("vote", params)
	}

	async fn cancel_vote(&self, voter: &H160) -> Result<TransactionBuilder<T>, ContractError> {
		self.vote(voter, None).await
	}

	fn build_vote_script(
		&self,
		voter: &H160,
		candidate: Option<&PublicKey>,
	) -> Result<Vec<u8>, ContractError> {
		let params = match candidate {
			Some(key) => vec![voter.into(), key.to_stack_item()],
			None => vec![voter.into(), StackItem::null()],
		};

		self.build_invoke_function_script("vote", params)
	}

	// Network Settings

	async fn get_gas_per_block(&self) -> Result<i32, ContractError> {
		self.call_function_returning_int("getGasPerBlock", vec![]).await
	}

	fn set_gas_per_block(
		&self,
		gas_per_block: i32,
	) -> Result<TransactionBuilder<T>, ContractError> {
		self.invoke_function("setGasPerBlock", vec![gas_per_block.into()])
	}

	async fn get_register_price(&self) -> Result<i32, ContractError> {
		self.call_function_returning_int("getRegisterPrice", vec![]).await
	}

	fn set_register_price(
		&self,
		register_price: i32,
	) -> Result<TransactionBuilder<T>, ContractError> {
		self.invoke_function("setRegisterPrice", vec![register_price.into()])
	}

	async fn get_account_state(&self, account: &H160) -> Result<AccountState, ContractError> {
		let result = self
			.call_invoke_function("getAccountState", vec![account.into()], vec![])
			.await?
			.get_result()
			.stack
			.pop()
			.unwrap();

		match result {
			StackItem::Any => Ok(AccountState::with_no_balance()),
			StackItem::Array(items) if items.len() >= 3 => {
				let balance = items[0].as_i64()?;
				let update_height = items[1].as_i64()?;
				let public_key = items[2].as_public_key().cloned();

				Ok(AccountState { balance, balance_height: update_height, public_key })
			},
			_ => Err(ContractError::InvalidNeoName("Account state malformed".to_string())),
		}
	}
}

#[async_trait]
impl<T> TokenTrait<T> for NeoToken {
	fn total_supply(&self) -> Option<u64> {
		self.total_supply
	}

	fn set_total_supply(&mut self, total_supply: u64) {
		self.total_supply = Some(total_supply)
	}

	fn decimals(&self) -> Option<u8> {
		self.decimals
	}

	fn set_decimals(&mut self, decimals: u8) {
		self.decimals = Some(decimals)
	}

	fn symbol(&self) -> Option<String> {
		self.symbol.clone()
	}

	fn set_symbol(&mut self, symbol: String) {
		self.symbol = Some(symbol)
	}
}

#[async_trait]
impl<T> SmartContractTrait<T> for NeoToken {
	fn script_hash(&self) -> H160 {
		self.script_hash
	}

	fn set_script_hash(&mut self, script_hash: H160) {
		self.script_hash = script_hash;
	}
}

#[async_trait]
impl<T> FungibleTokenTrait<T> for NeoToken {}

pub struct Candidate {
	pub public_key: PublicKey,
	pub votes: i32,
}

impl Candidate {
	fn from(items: Vec<StackItem>) -> Result<Self, ContractError> {
		let key = items[0].as_public_key()?;
		let votes = items[1].as_i32()?;
		Ok(Self { public_key: key, votes })
	}
}
