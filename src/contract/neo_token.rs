use crate::{
	contract::{
		contract_error::ContractError,
		traits::{
			fungible_token::FungibleTokenTrait, smartcontract::SmartContractTrait,
			token::TokenTrait,
		},
	},
	neo_error::NeoError,
	protocol::core::{responses::neo_account_state::AccountState, stack_item::StackItem},
	transaction::transaction_builder::TransactionBuilder,
	types::{
		contract_parameter::ContractParameter, contract_parameter_type::ContractParameterType,
		PublicKey,
	},
	utils::*,
	wallet::account::Account,
};
use async_trait::async_trait;
use primitive_types::H160;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeoToken {
	#[serde(deserialize_with = "deserialize_address")]
	#[serde(serialize_with = "serialize_address")]
	script_hash: H160,
	#[serde(skip_serializing_if = "Option::is_none")]
	total_supply: Option<u64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	decimals: Option<u8>,
	symbol: Option<String>,
}

impl NeoToken {
	pub const NAME: &'static str = "NeoToken";
	// pub const SCRIPT_HASH: H160 = Self::calc_native_contract_hash(Self::NAME).unwrap();
	pub const DECIMALS: u8 = 0;
	pub const SYMBOL: &'static str = "NEO";
	pub const TOTAL_SUPPLY: u64 = 100_000_000;

	pub(crate) fn new() -> Self {
		NeoToken {
			script_hash: Self::calc_native_contract_hash(Self::NAME).unwrap(),
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
		self.unclaimed_gas_contract(&account.get_script_hash(), block_height).await
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

	async fn register_candidate(
		&self,
		candidate_key: &PublicKey,
	) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("registerCandidate", vec![candidate_key.into()]).await
	}

	async fn unregister_candidate(
		&self,
		candidate_key: &PublicKey,
	) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("unregisterCandidate", vec![candidate_key.into()]).await
	}

	// Committee and Candidates Information

	async fn get_committee(&self) -> Result<Vec<PublicKey>, ContractError> {
		self.call_function_returning_list_of_public_keys("getCommittee")
			.await
			.map_err(|e| ContractError::UnexpectedReturnType(e.to_string()))
	}

	async fn get_candidates(&self) -> Result<Vec<Candidate>, ContractError> {
		let candidates = self.call_invoke_function("getCandidates", vec![], vec![]).await.unwrap();
		let item = candidates.stack.first().unwrap();
		if let StackItem::Array { value: array } = item {
			Ok(array
				.to_vec()
				.chunks(2)
				.filter_map(|v| {
					if v.len() == 2 {
						Some(Candidate::from(v.to_vec()).unwrap())
					} else {
						None
					}
				})
				.collect::<Vec<Candidate>>())
		} else {
			Err(ContractError::UnexpectedReturnType("Candidates".to_string()))
		}
	}

	async fn is_candidate(&self, public_key: &PublicKey) -> Result<bool, ContractError> {
		Ok(self
			.get_candidates()
			.await
			.unwrap()
			.into_iter()
			.any(|c| c.public_key == *public_key))
	}

	// Voting

	async fn vote(
		&self,
		voter: &H160,
		candidate: Option<&PublicKey>,
	) -> Result<TransactionBuilder, ContractError> {
		let params = match candidate {
			Some(key) => vec![voter.into(), key.into()],
			None => vec![voter.into(), ContractParameter::new(ContractParameterType::Any)],
		};

		self.invoke_function("vote", params).await
	}

	async fn cancel_vote(&self, voter: &H160) -> Result<TransactionBuilder, ContractError> {
		self.vote(voter, None).await
	}

	async fn build_vote_script(
		&self,
		voter: &H160,
		candidate: Option<&PublicKey>,
	) -> Result<Vec<u8>, ContractError> {
		let params = match candidate {
			Some(key) => vec![voter.into(), key.into()],
			None => vec![voter.into(), ContractParameter::new(ContractParameterType::Any)],
		};

		self.build_invoke_function_script("vote", params).await
	}

	// Network Settings

	async fn get_gas_per_block(&self) -> Result<i32, ContractError> {
		self.call_function_returning_int("getGasPerBlock", vec![]).await
	}

	async fn set_gas_per_block(
		&self,
		gas_per_block: i32,
	) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("setGasPerBlock", vec![gas_per_block.into()]).await
	}

	async fn get_register_price(&self) -> Result<i32, ContractError> {
		self.call_function_returning_int("getRegisterPrice", vec![]).await
	}

	async fn set_register_price(
		&self,
		register_price: i32,
	) -> Result<TransactionBuilder, ContractError> {
		self.invoke_function("setRegisterPrice", vec![register_price.into()]).await
	}

	async fn get_account_state(&self, account: &H160) -> Result<AccountState, ContractError> {
		let result = self
			.call_invoke_function("getAccountState", vec![account.into()], vec![])
			.await
			.unwrap()
			.stack
			.first()
			.unwrap()
			.clone();

		match result {
			StackItem::Any => Ok(AccountState::with_no_balance()),
			StackItem::Array { value: items } if items.len() >= 3 => {
				let balance = items[0].as_int().unwrap();
				let update_height = items[1].as_int();
				let public_key = items[2].clone();

				if let StackItem::Any = public_key {
					return Ok(AccountState {
						balance,
						balance_height: update_height,
						public_key: None,
					})
				} else {
					let pubkey =
						PublicKey::from_sec1_bytes(public_key.as_bytes().unwrap().as_slice())
							.unwrap();
					Ok(AccountState {
						balance,
						balance_height: update_height,
						public_key: Some(pubkey),
					})
				}
			},
			_ => Err(ContractError::InvalidNeoName("Account state malformed".to_string())),
		}
	}

	async fn call_function_returning_list_of_public_keys(
		&self,
		function: &str,
	) -> Result<Vec<PublicKey>, NeoError> {
		let result = self.call_invoke_function(function, vec![], vec![]).await.unwrap();
		let stack_item = result.stack.first().unwrap();

		if let StackItem::Array { value: array } = stack_item {
			let keys = array
				.iter()
				.map(|item| {
					if let StackItem::ByteString { value: bytes } = item {
						PublicKey::from_sec1_bytes(bytes.as_bytes())
							.map_err(|_| NeoError::UnexpectedReturnType)
					} else {
						Err(NeoError::UnexpectedReturnType)
					}
				})
				.collect::<Result<Vec<PublicKey>, NeoError>>()?;

			Ok(keys)
		} else {
			Err(NeoError::UnexpectedReturnType)
		}
	}
}

#[async_trait]
impl TokenTrait for NeoToken {
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
impl SmartContractTrait for NeoToken {
	fn script_hash(&self) -> H160 {
		self.script_hash
	}

	fn set_script_hash(&mut self, script_hash: H160) {
		self.script_hash = script_hash;
	}
}

#[async_trait]
impl FungibleTokenTrait for NeoToken {}

pub struct Candidate {
	pub public_key: PublicKey,
	pub votes: i32,
}

impl Candidate {
	fn from(items: Vec<StackItem>) -> Result<Self, ContractError> {
		let key = items[0].as_public_key().unwrap();
		let votes = items[1].as_int().unwrap() as i32;
		Ok(Self { public_key: key, votes })
	}
}
