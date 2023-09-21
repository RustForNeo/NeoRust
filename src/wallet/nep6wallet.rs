use crate::wallet::nep6account::NEP6Account;
use crypto::scrypt::ScryptParams;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Getters, CopyGetters)]
#[getset(get = "pub", set = "pub")]
pub struct NEP6Wallet {
	name: String,
	version: String,
	scrypt: ScryptParams,
	accounts: Vec<NEP6Account>,
	extra: Option<HashMap<String, String>>,
}

impl NEP6Wallet {
	pub fn new(
		name: String,
		version: String,
		scrypt: ScryptParams,
		accounts: Vec<NEP6Account>,
		extra: Option<HashMap<String, String>>,
	) -> Self {
		Self { name, version, scrypt, accounts, extra }
	}
}
