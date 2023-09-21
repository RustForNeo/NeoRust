use crate::{
	protocol::core::witness_rule::witness_rule::WitnessRule,
	transaction::{
		signer::{Signer, SignerTrait, SignerType},
		transaction_error::TransactionError,
		witness_scope::WitnessScope,
	},
	types::{H160Externsion, PublicKey},
	utils::*,
	wallet::account::Account,
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSigner {
	#[serde(serialize_with = "serialize_address", deserialize_with = "deserialize_address")]
	signer_hash: H160,
	scopes: Vec<WitnessScope>,
	#[serde(
		serialize_with = "serialize_vec_address",
		deserialize_with = "deserialize_vec_address"
	)]
	allowed_contracts: Vec<H160>,
	#[serde(
		serialize_with = "serialize_vec_public_key",
		deserialize_with = "deserialize_vec_public_key"
	)]
	allowed_groups: Vec<PublicKey>,
	rules: Vec<WitnessRule>,

	pub account: Account,
	scope: WitnessScope,
}

impl SignerTrait for AccountSigner {
	fn get_type(&self) -> SignerType {
		SignerType::Account
	}

	fn get_signer_hash(&self) -> &H160 {
		&self.signer_hash
	}

	fn set_signer_hash(&mut self, signer_hash: H160) {
		self.signer_hash = signer_hash;
	}

	fn get_scopes(&self) -> &Vec<WitnessScope> {
		&self.scopes
	}

	fn set_scopes(&mut self, scopes: Vec<WitnessScope>) {
		self.scopes = scopes;
	}

	fn get_allowed_contracts(&self) -> &Vec<H160> {
		&self.allowed_contracts
	}

	fn get_allowed_groups(&self) -> &Vec<PublicKey> {
		&self.allowed_groups
	}

	fn get_rules(&self) -> &Vec<WitnessRule> {
		&self.rules
	}
}

impl AccountSigner {
	fn new(account: &Account, scope: WitnessScope) -> Self {
		Self {
			signer_hash: account.get_script_hash().unwrap(),
			scopes: vec![],
			allowed_contracts: vec![],
			allowed_groups: vec![],
			rules: vec![],
			account: account.clone(),
			scope,
		}
	}

	pub fn none(account: &Account) -> Result<Self, TransactionError> {
		Ok(Self::new(account, WitnessScope::None))
	}

	pub fn none_hash160(account_hash: H160) -> Result<Self, TransactionError> {
		let account = Account::from_address(account_hash.to_address().as_str()).unwrap();
		Ok(Self::new(&account, WitnessScope::None))
	}

	pub fn called_by_entry(account: &Account) -> Result<Self, TransactionError> {
		Ok(Self::new(account, WitnessScope::CalledByEntry))
	}

	pub fn called_by_entry_hash160(account_hash: H160) -> Result<Self, TransactionError> {
		let account = Account::from_address(account_hash.to_address().as_str()).unwrap();
		Ok(Self::new(&account, WitnessScope::CalledByEntry))
	}

	pub fn global(account: Account) -> Result<Self, TransactionError> {
		Ok(Self::new(&account, WitnessScope::Global))
	}

	pub fn global_hash160(account_hash: H160) -> Result<Self, TransactionError> {
		let account = Account::from_address(account_hash.to_address().as_str()).unwrap();
		Ok(Self::new(&account, WitnessScope::Global))
	}
}
