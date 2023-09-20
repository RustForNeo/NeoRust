use crate::{
	protocol::core::witness_rule::witness_rule::WitnessRule,
	transaction::witness_scope::WitnessScope,
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TransactionSigner {
	#[serde(rename = "account")]
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
