use crate::{types::ScryptParamsDef, wallet::nep6account::NEP6Account};
use crypto::scrypt::ScryptParams;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Getters, CopyGetters)]
#[getset(get = "pub", set = "pub")]
pub struct NEP6Wallet {
	pub(crate) name: String,
	pub(crate) version: String,
	#[serde(skip_serializing)]
	pub(crate) scrypt: ScryptParamsDef,
	pub(crate) accounts: Vec<NEP6Account>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub(crate) extra: Option<HashMap<String, String>>,
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
