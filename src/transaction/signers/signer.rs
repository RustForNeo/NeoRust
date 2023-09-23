use crate::{
	constant::NeoConstants,
	neo_error::NeoError,
	protocol::core::witness_rule::{
		witness_condition::WitnessCondition, witness_rule::WitnessRule,
	},
	transaction::{
		signers::account_signer::AccountSigner, signers::contract_signer::ContractSigner, witness_scope::WitnessScope,
	},
	types::PublicKey,
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use crate::transaction::signers::transaction_signer::TransactionSigner;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SignerType {
	Account,
	Contract,
	Transaction,
}

pub trait SignerTrait {
	fn get_type(&self) -> SignerType;

	fn get_signer_hash(&self) -> &H160;

	fn set_signer_hash(&mut self, signer_hash: H160);

	fn get_scopes(&self) -> &Vec<WitnessScope>;

	fn set_scopes(&mut self, scopes: Vec<WitnessScope>);

	fn get_allowed_contracts(&self) -> &Vec<H160>;

	// fn set_allowed_contracts(&mut self, allowed_contracts: Vec<H160>);

	fn get_allowed_groups(&self) -> &Vec<PublicKey>;

	fn get_rules(&self) -> &Vec<WitnessRule>;

	// fn new(signer_hash: H160, scope: WitnessScope) -> Self {
	//     Self {
	//         signer_hash,
	//         scopes: vec![scope],
	//         allowed_contracts: Vec::new(),
	//         allowed_groups: Vec::new(),
	//         rules: Vec::new(),
	//     }
	// }

	// Setters

	// Set allowed contracts
	fn set_allowed_contracts(&mut self, contracts: Vec<H160>) -> Result<(), NeoError> {
		// Validate
		if self.get_scopes().contains(&WitnessScope::Global) {
			return Err(NeoError::InvalidConfiguration(
				"Cannot set contracts for global scope".to_string(),
			))
		}

		if self.get_allowed_contracts().len() + contracts.len()
			> NeoConstants::MAX_SIGNER_SUBITEMS as usize
		{
			return Err(NeoError::InvalidConfiguration("Too many allowed contracts".to_string()))
		}

		// Update state
		if !self.get_scopes().contains(&WitnessScope::CustomContracts) {
			self.get_scopes().push(WitnessScope::CustomContracts);
		}

		self.get_allowed_contracts().extend(contracts);

		Ok(())
	}

	// Set allowed groups
	fn set_allowed_groups(&mut self, groups: Vec<PublicKey>) -> Result<(), NeoError> {
		if self.get_scopes().contains(&WitnessScope::Global) {
			return Err(NeoError::InvalidConfiguration(
				"Cannot set groups for global scope".to_string(),
			))
		}

		if self.get_allowed_groups().len() + groups.len()
			> NeoConstants::MAX_SIGNER_SUBITEMS as usize
		{
			return Err(NeoError::InvalidConfiguration("Too many allowed groups".to_string()))
		}

		if !self.get_scopes().contains(&WitnessScope::CustomGroups) {
			self.get_scopes().push(WitnessScope::CustomGroups);
		}

		self.get_allowed_groups().extend(groups);

		Ok(())
	}

	// Set rules
	fn set_rules(&mut self, rules: Vec<WitnessRule>) -> Result<(), NeoError> {
		if self.get_scopes().contains(&WitnessScope::Global) {
			return Err(NeoError::InvalidConfiguration(
				"Cannot set rules for global scope".to_string(),
			))
		}

		if self.get_rules().len() + rules.len() > NeoConstants::MAX_SIGNER_SUBITEMS as usize {
			return Err(NeoError::InvalidConfiguration("Too many rules".to_string()))
		}

		// Validate nesting depth
		for rule in &rules {
			self.validate_depth(&rule.condition, NeoConstants::MAX_NESTING_DEPTH).unwrap();
		}

		if !self.get_scopes().contains(&WitnessScope::WitnessRules) {
			self.get_scopes().push(WitnessScope::WitnessRules);
		}

		self.get_rules().extend(rules);

		Ok(())
	}

	// Check depth recursively
	fn validate_depth(&self, rule: &WitnessCondition, depth: u8) -> Result<(), NeoError> {
		// Depth exceeded
		if depth == 0 {
			return Err(NeoError::InvalidConfiguration("Max nesting depth exceeded".to_string()))
		}

		match &rule {
			WitnessCondition::And(conditions) | WitnessCondition::Or(conditions) => {
				for inner_rule in conditions {
					self.validate_depth(inner_rule, depth - 1).unwrap();
				}
			},
			_ => (),
		}

		Ok(())
	}
	fn validate_subitems(&self, count: usize, name: &str) -> Result<(), NeoError> {
		if count > NeoConstants::MAX_SIGNER_SUBITEMS as usize {
			return Err(NeoError::InvalidData(format!("Too many {} in signer", name)))
		}
		Ok(())
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Signer {
	Account(AccountSigner),
	Contract(ContractSigner),
	Transaction(TransactionSigner),
}

impl Signer {
	pub fn get_type(&self) -> SignerType {
		match self {
			Signer::Account(account_signer) => account_signer.get_type(),
			Signer::Contract(contract_signer) => contract_signer.get_type(),
			Signer::Transaction(transaction_signer) => transaction_signer.get_type(),
		}
	}
	pub fn get_signer_hash(&self) -> &H160 {
		match self {
			Signer::Account(account_signer) => account_signer.get_signer_hash(),
			Signer::Contract(contract_signer) => contract_signer.get_signer_hash(),
			Signer::Transaction(transaction_signer) => transaction_signer.get_signer_hash(),
		}
	}

	pub fn as_account_signer(&self) -> Option<&AccountSigner> {
		match self {
			Signer::Account(account_signer) => Some(account_signer),
			_ => None,

		}
	}

	pub fn as_contract_signer(&self) -> Option<&ContractSigner> {
		match self {
			Signer::Contract(contract_signer) => Some(contract_signer),
			_ => None,
		}
	}

	pub fn as_transaction_signer(&self) -> Option<&TransactionSigner> {
		match self {
			Signer::Transaction(transaction_signer) => Some(transaction_signer),
			_ => None,
		}
	}
}

impl Hash for Signer {
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self {
			Signer::Account(account_signer) => account_signer.hash(state),
			Signer::Contract(contract_signer) => contract_signer.hash(state),
			Signer::Transaction(transaction_signer) => transaction_signer.hash(state),
		}
	}
}

impl From<AccountSigner> for Signer {
	fn from(account_signer: AccountSigner) -> Self {
		Signer::Account(account_signer)
	}
}

impl From<ContractSigner> for Signer {
	fn from(contract_signer: ContractSigner) -> Self {
		Signer::Contract(contract_signer)
	}
}

impl Into<AccountSigner> for Signer {
	fn into(self) -> AccountSigner {
		match self {
			Signer::Account(account_signer) => account_signer,
			_ =>
				panic!("Cannot convert ContractSigner into AccountSigner"),
		}
	}
}

impl Into<TransactionSigner> for Signer {
	fn into(self) -> TransactionSigner {
		match self {
			Signer::Account(account_signer) => panic!("Cannot convert AccountSigner into TransactionSigner"),
			Signer::Contract(contract_signer) =>
				panic!("Cannot convert ContractSigner into AccountSigner"),
			Signer::Transaction(transaction_signer) => transaction_signer,
		}
	}
}

impl Into<TransactionSigner> for &Signer{
	fn into(self) -> TransactionSigner {
		match self {
			Signer::Account(account_signer) => panic!("Cannot convert AccountSigner into TransactionSigner"),
			Signer::Contract(contract_signer) =>
				panic!("Cannot convert ContractSigner into AccountSigner"),
			Signer::Transaction(transaction_signer) => transaction_signer.clone(),
		}
	}
}

impl Into<TransactionSigner> for &mut Signer{
	fn into(self) -> TransactionSigner {
		match self {
			Signer::Account(account_signer) => panic!("Cannot convert AccountSigner into TransactionSigner"),
			Signer::Contract(contract_signer) =>
				panic!("Cannot convert ContractSigner into AccountSigner"),
			Signer::Transaction(transaction_signer) => transaction_signer.clone(),
		}
	}
}

impl Into<AccountSigner> for &mut Signer {
	fn into(self) -> AccountSigner {
		match self {
			Signer::Account(account_signer) => account_signer.clone(),
			Signer::Contract(contract_signer) =>
				panic!("Cannot convert ContractSigner into AccountSigner"),
			Signer::Transaction(transaction_signer) =>
				panic!("Cannot convert TransactionSigner into AccountSigner"),
		}
	}
}

impl Into<ContractSigner> for &mut Signer {
	fn into(self) -> ContractSigner {
		match self {
			Signer::Account(account_signer) =>
				panic!("Cannot convert AccountSigner into ContractSigner"),
			Signer::Contract(contract_signer) => contract_signer.clone(),
			Signer::Transaction(transaction_signer) =>
				panic!("Cannot convert TransactionSigner into ContractSigner"),
		}
	}
}

impl Into<ContractSigner> for Signer {
	fn into(self) -> ContractSigner {
		match self {
			Signer::Account(account_signer) =>
				panic!("Cannot convert AccountSigner into ContractSigner"),
			Signer::Contract(contract_signer) => contract_signer,
			Signer::Transaction(transaction_signer) =>
				panic!("Cannot convert TransactionSigner into ContractSigner"),
		}
	}
}
