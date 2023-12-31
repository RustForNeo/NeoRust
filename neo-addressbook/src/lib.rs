#![doc = include_str!("../README.md")]
#![deny(unsafe_code, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use neo_config::NeoNetwork;
use neo_types::address::Address;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const CONTRACTS_JSON: &str = include_str!("./contracts/contracts.json");

static ADDRESSBOOK: Lazy<HashMap<String, Contract>> =
	Lazy::new(|| serde_json::from_str(CONTRACTS_JSON).unwrap());

/// Wrapper around a hash map that maps a [Chain] to the contract's deployed address on that chain.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Contract {
	addresses: HashMap<NeoNetwork, Address>,
}

impl Contract {
	/// Returns the address of the contract on the specified chain. If the contract's address is
	/// not found in the addressbook, the getter returns None.
	pub fn address(&self, network: NeoNetwork) -> Option<Address> {
		self.addresses.get(&network).cloned()
	}
}

/// Fetch the addressbook for a contract by its name. If the contract name is not a part of
pub fn contract<S: Into<String>>(name: S) -> Option<Contract> {
	ADDRESSBOOK.get(&name.into()).cloned()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_tokens() {
		assert!(contract("dai").is_some());
		assert!(contract("usdc").is_some());
		assert!(contract("usdt").is_some());
		assert!(contract("rand").is_none());
	}

	#[test]
	fn test_addrs() {
		assert!(contract("dai").unwrap().address(NeoNetwork::MainNet).is_some());
	}
}
