use crate::{
	protocol::core::witness_rule::witness_rule::WitnessRule,
	transaction::{
		signer::{SignerTrait, SignerType},
		witness_scope::WitnessScope,
	},
	types::{contract_parameter::ContractParameter, PublicKey},
	utils::*,
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct ContractSigner {
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
	pub verify_params: Vec<ContractParameter>,
	#[serde(serialize_with = "serialize_address", deserialize_with = "deserialize_address")]
	contract_hash: H160,
	scope: WitnessScope,
}

impl Hash for ContractSigner {
	fn hash<H: Hasher>(&self, state: &mut H) {
		// self.signer_hash.hash(state);
		self.scopes.hash(state);
		// self.allowed_contracts.hash(state);
		// self.allowed_groups.hash(state);
		self.rules.hash(state);
		self.verify_params.hash(state);
		self.contract_hash.hash(state);
		self.scope.hash(state);
	}
}

impl SignerTrait for ContractSigner {
	fn get_type(&self) -> SignerType {
		SignerType::Contract
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

impl ContractSigner {
	fn new(
		contract_hash: H160,
		scope: WitnessScope,
		verify_params: Vec<ContractParameter>,
	) -> Self {
		Self {
			signer_hash: Default::default(),
			scopes: vec![],
			allowed_contracts: vec![],
			allowed_groups: vec![],
			rules: vec![],
			verify_params,
			contract_hash,
			scope,
		}
	}

	pub fn called_by_entry(contract_hash: H160, verify_params: &[ContractParameter]) -> Self {
		Self::new(contract_hash, WitnessScope::CalledByEntry, verify_params.to_vec())
	}

	pub fn global(contract_hash: H160, verify_params: &[ContractParameter]) -> Self {
		Self::new(contract_hash, WitnessScope::Global, verify_params.to_vec())
	}
}
