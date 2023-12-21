use crate::transaction::{
	signers::signer::{SignerTrait, SignerType},
	witness_rule::witness_rule::WitnessRule,
	witness_scope::WitnessScope,
};
use neo_types::*;

use neo_crypto::keys::Secp256r1PublicKey;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::{
	hash::{Hash, Hasher},
	str::FromStr,
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct TransactionSigner {
	#[serde(rename = "account")]
	#[serde(serialize_with = "serialize_script_hash")]
	#[serde(deserialize_with = "deserialize_script_hash")]
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

impl SignerTrait for TransactionSigner {
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
		panic!("Not implemented")
	}

	fn get_allowed_contracts_mut(&mut self) -> &mut Vec<H160> {
		panic!("Not implemented")
	}

	fn get_allowed_groups(&self) -> &Vec<Secp256r1PublicKey> {
		panic!("Not implemented")
		// &self.allowed_groups
	}

	fn get_allowed_groups_mut(&mut self) -> &mut Vec<Secp256r1PublicKey> {
		panic!("Not implemented")
	}

	fn get_rules(&self) -> &Vec<WitnessRule> {
		panic!("Not implemented")
	}

	fn get_rules_mut(&mut self) -> &mut Vec<WitnessRule> {
		panic!("Not implemented")
	}
}
