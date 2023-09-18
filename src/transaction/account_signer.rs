use crate::{
	protocol::core::witness_rule::witness_rule::WitnessRule,
	transaction::{
		signer::Signer, transaction_error::TransactionError, witness_scope::WitnessScope,
	},
	types::H160Externsion,
	wallet::account::Account,
};
use p256::PublicKey;
use primitive_types::H160;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct AccountSigner {
	signer_hash: H160,
	scopes: Vec<WitnessScope>,
	allowed_contracts: Vec<H160>,
	allowed_groups: Vec<PublicKey>,
	rules: Vec<WitnessRule>,
	pub account: Account,
	scope: WitnessScope,
}

impl Signer for AccountSigner {
	type SignerType = AccountSigner;

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
}

impl AccountSigner {
	fn new(account: &Account, scope: WitnessScope) -> Self {
		Self {
			signer_hash: account.getScriptHash(),
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
