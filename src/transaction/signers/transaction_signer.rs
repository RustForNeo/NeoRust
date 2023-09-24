use crate::{
	protocol::core::witness_rule::witness_rule::WitnessRule,
	transaction::witness_scope::WitnessScope, utils::*,
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use crate::transaction::signers::signer::{Signer, SignerTrait, SignerType};
use crate::types::{PublicKey, PublicKeyExtension};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct TransactionSigner {
	#[serde(rename = "account")]
	#[serde(serialize_with = "serialize_address")]
	#[serde(deserialize_with = "deserialize_address")]
	pub account: H160,

	#[serde(rename = "scopes")]
	pub scopes: Vec<WitnessScope>,

	#[serde(rename = "allowedcontracts")]
	pub allowed_contracts: Option<Vec<String>>,

	#[serde(rename = "allowedgroups")]
	pub allowed_groups: Option<Vec<String>>,

	#[serde(rename = "rules")]
	pub rules: Option<Vec<WitnessRule>>,
}

impl TransactionSigner {
	pub fn new(account: H160, scopes: Vec<WitnessScope>) -> Self {
		Self { account, scopes, allowed_contracts: None, allowed_groups: None, rules: None }
	}

	pub fn new_full(
		account: H160,
		scopes: Vec<WitnessScope>,
		allowed_contracts: Vec<String>,
		allowed_groups: Vec<String>,
		rules: Vec<WitnessRule>,
	) -> Self {
		Self {
			account,
			scopes,
			allowed_contracts: Some(allowed_contracts),
			allowed_groups: Some(allowed_groups),
			rules: Some(rules),
		}
	}
}

impl SignerTrait for TransactionSigner{
	fn get_type(&self) -> SignerType {
		SignerType::Transaction
	}

	fn get_signer_hash(&self) -> &H160 {
		&self.account
	}

	fn set_signer_hash(&mut self, signer_hash: H160) {
		self.account = signer_hash;
	}

	fn get_scopes(&self) -> &Vec<WitnessScope> {
		&self.scopes
	}

	fn get_scopes_mut(&mut self) -> &mut Vec<WitnessScope> {
		&mut self.scopes
	}

	fn set_scopes(&mut self, scopes: Vec<WitnessScope>) {
		self.scopes = scopes;
	}

	fn get_allowed_contracts(&self) -> &Vec<H160> {
		&self.allowed_contracts
			.clone()
			.map(|x| x.iter()
				.map(|y| H160::from_str(y).unwrap())
				.collect::<Vec<_>>())
			.unwrap_or_else(Vec::new)
	}

	fn get_allowed_contracts_mut(&mut self) -> &mut Vec<H160> {
		&mut self.allowed_contracts
			.clone()
			.map(|x| x.iter()
				.map(|y| H160::from_str(y).unwrap())
				.collect::<Vec<_>>())
			.unwrap_or_else(Vec::new)
	}

	fn get_allowed_groups(&self) -> &Vec<PublicKey> {
		panic!("Not implemented")
		// &self.allowed_groups
	}

	fn get_allowed_groups_mut(&mut self) -> &mut Vec<PublicKey> {
		&mut self.allowed_groups
			.clone()
			.map(|x| x.iter()
				.map(|y| PublicKey::from_hex(y).unwrap())
				.collect::<Vec<_>>())
			.unwrap_or_else(Vec::new)
	}

	fn get_rules(&self) -> &Vec<WitnessRule> {
		&self.rules.unwrap()
	}

	fn get_rules_mut(&mut self) -> &mut Vec<WitnessRule> {
		&mut self.rules.unwrap()
	}
}