use crate::wallet::nep6account::NEP6Account;
use crypto::scrypt::ScryptParams;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[macro_use]
extern crate getset;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Getters, CopyGetters, Default)]
#[getset(get = "pub", set = "pub")]
pub struct NEP6Wallet {
	name: String,
	version: String,
	scrypt: ScryptParams,
	accounts: Vec<NEP6Account>,
	extra: Option<HashMap<String, String>>,
}

impl PartialEq for NEP6Wallet {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
			&& self.version == other.version
			&& self.scrypt == other.scrypt
			&& self.extra == other.extra
			&& self.accounts.len() == other.accounts.len()
			&& self.accounts.iter().all(|acc| other.accounts.contains(acc))
	}
}
