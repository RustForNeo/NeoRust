use crate::{
	contract::{
		contract_error::ContractError, fungible_token::FungibleToken, smartcontract::SmartContract,
	},
	protocol::core::{responses::neo_account_state::AccountState, stack_item::StackItem},
	transaction::transaction_builder::TransactionBuilder,
	types::ECPublicKey,
	wallet::account::Account,
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeoToken {
	script_hash: H160,
}

impl NeoToken {
	pub const NAME: &'static str = "NeoToken";
	pub const SCRIPT_HASH: H160 = SmartContract::calc_native_contract_hash(Self::NAME).unwrap();
	pub const DECIMALS: u8 = 0;
	pub const SYMBOL: &'static str = "NEO";
	pub const TOTAL_SUPPLY: u64 = 100_000_000;

	fn new() -> Self {
		NeoToken { script_hash: Self::SCRIPT_HASH }
	}

	async fn get_name(&self) -> Result<Option<String>, ContractError> {
		Ok(Some(Self::NAME.to_string()))
	}

	async fn get_symbol(&self) -> Result<String, ContractError> {
		Ok(Self::SYMBOL.to_string())
	}

	async fn get_decimals(&self) -> Result<u8, ContractError> {
		Ok(Self::DECIMALS)
	}

	async fn get_total_supply(&self) -> Result<u64, ContractError> {
		Ok(Self::TOTAL_SUPPLY)
	}

	// Unclaimed Gas

	async fn unclaimed_gas(
		&self,
		account: &Account,
		block_height: i32,
	) -> Result<i64, ContractError> {
		self.unclaimed_gas(account.get_script_hash().unwrap(), block_height).await
	}

	async fn unclaimed_gas_contract(
		&self,
		script_hash: &H160,
		block_height: i32,
	) -> Result<i64, ContractError> {
		self.call_function_returning_int(
			"unclaimedGas",
			vec![script_hash.into(), block_height.into()],
		)
		.await
	}

	// Candidate Registration

	fn register_candidate(
		&self,
		candidate_key: &ECPublicKey,
	) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("registerCandidate", vec![candidate_key.to_stack_item()])
	}

	fn unregister_candidate(
		&self,
		candidate_key: &ECPublicKey,
	) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("unregisterCandidate", vec![candidate_key.to_stack_item()])
	}

	// Committee and Candidates Information

	async fn get_committee(&self) -> Result<Vec<ECPublicKey>, ContractError> {
		self.call_function_returning_list_of_keys("getCommittee").await
	}

	async fn get_candidates(&self) -> Result<Vec<Candidate>, ContractError> {
		let candidates = self.call_function_returning_candidates("getCandidates").await?;
		candidates.into_iter().map(Candidate::from).collect()
	}

	async fn is_candidate(&self, public_key: &ECPublicKey) -> Result<bool, ContractError> {
		Ok(self.get_candidates().await?.into_iter().any(|c| c.public_key == *public_key))
	}

	// Voting

	async fn vote(
		&self,
		voter: &H160,
		candidate: Option<&ECPublicKey>,
	) -> Result<TransactionBuilder, ContractError> {
		let params = match candidate {
			Some(key) => vec![voter.into(), key.to_stack_item()],
			None => vec![voter.into(), StackItem::null()],
		};

		self.invoke_function("vote", params)
	}

	async fn cancel_vote(&self, voter: &H160) -> Result<TransactionBuilder, ContractError> {
		self.vote(voter, None).await
	}

	fn build_vote_script(
		&self,
		voter: &H160,
		candidate: Option<&ECPublicKey>,
	) -> Result<Vec<u8>, ContractError> {
		let params = match candidate {
			Some(key) => vec![voter.into(), key.to_stack_item()],
			None => vec![voter.into(), StackItem::null()],
		};

		self.build_invoke_function_script("vote", params)
	}

	// Network Settings

	async fn get_gas_per_block(&self) -> Result<i32, ContractError> {
		self.call_function_returning_int("getGasPerBlock").await
	}

	fn set_gas_per_block(&self, gas_per_block: i32) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("setGasPerBlock", vec![gas_per_block.into()])
	}

	async fn get_register_price(&self) -> Result<i32, ContractError> {
		self.call_function_returning_int("getRegisterPrice").await
	}

	fn set_register_price(&self, register_price: i32) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("setRegisterPrice", vec![register_price.into()])
	}

	async fn get_account_state(&self, account: &H160) -> Result<AccountState, ContractError> {
		let result = self
			.call_invoke_function("getAccountState", vec![account.into()])
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

impl FungibleToken for NeoToken {}

pub struct Candidate {
	pub public_key: ECPublicKey,
	pub votes: i32,
}

impl Candidate {
	fn from(items: Vec<StackItem>) -> Result<Self, ContractError> {
		let key = items[0].as_public_key()?;
		let votes = items[1].as_i32()?;
		Ok(Self { public_key: key, votes })
	}
}
