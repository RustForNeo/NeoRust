use crate::wallet::{account::Account, nep6wallet::NEP6Wallet, wallet_error::WalletError};
use neo_providers::core::{account::AccountTrait, wallet::WalletTrait};
use neo_types::{address::AddressExtension, ScryptParamsDef, *};
use primitive_types::H160;
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
	name: String,
	version: String,
	scrypt_params: ScryptParamsDef,
	#[serde(deserialize_with = "deserialize_hash_map_h160_account")]
	#[serde(serialize_with = "serialize_hash_map_h160_account")]
	pub accounts: HashMap<H160, Account>,
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	default_account: H160,
}

impl WalletTrait for Wallet {
	type Account = Account;

	fn name(&self) -> &String {
		&self.name
	}

	fn version(&self) -> &String {
		&self.version
	}

	fn scrypt_params(&self) -> &ScryptParamsDef {
		&self.scrypt_params
	}

	fn accounts(&self) -> &HashMap<H160, Self::Account> {
		&self.accounts
	}

	fn default_account(&self) -> &H160 {
		&self.default_account
	}

	fn set_name(&mut self, name: String) {
		self.name = name;
	}

	fn set_version(&mut self, version: String) {
		self.version = version;
	}

	fn set_scrypt_params(&mut self, params: ScryptParamsDef) {
		self.scrypt_params = params;
	}

	fn set_default_account(&mut self, default_account: H160) {
		self.default_account = default_account;
	}

	fn add_account(&mut self, account: Self::Account) {
		self.accounts.insert(account.script_hash().clone(), account);
	}

	fn remove_account(&mut self, hash: &H160) -> Option<Self::Account> {
		self.accounts.remove(hash)
	}
}

impl Wallet {
	// Constructor

	pub fn new() -> Self {
		Self {
			name: "MyWallet".to_string(),
			version: "1.0".to_string(),
			scrypt_params: ScryptParamsDef::default(),
			accounts: HashMap::new(),
			default_account: H160::default(),
		}
	}

	// pub fn set_name(&mut self, name: &str) {
	// 	self.name = name.to_string();
	// }

	// pub fn add_account(&mut self, account: Account) {
	// 	self.accounts.insert(account.get_script_hash().clone(), account);
	// }

	// pub fn set_default_account(&mut self, script_hash: H160) {
	// 	self.default_account = script_hash;
	// }

	// Serialization methods

	pub fn to_nep6(&self) -> Result<NEP6Wallet, WalletError> {
		let accounts = self.accounts.values().filter_map(|a| a.to_nep6_account().ok()).collect();

		Ok(NEP6Wallet {
			name: self.name.clone(),
			version: self.version.clone(),
			scrypt: self.scrypt_params.clone(),
			accounts,
			extra: None,
		})
	}

	pub fn from_nep6(nep6: NEP6Wallet) -> Result<Self, WalletError> {
		let accounts = nep6
			.accounts()
			.into_iter()
			.filter_map(|v| Account::from_nep6_account(v).ok())
			.collect::<Vec<_>>();

		let default_account = nep6
			.accounts()
			.iter()
			.find(|a| a.is_default)
			.map(|a| a.address())
			.ok_or(WalletError::NoDefaultAccount)
			.unwrap();

		Ok(Self {
			name: nep6.name().clone(),
			version: nep6.version().clone(),
			scrypt_params: nep6.scrypt().clone(),
			accounts: accounts.into_iter().map(|a| (a.get_script_hash().clone(), a)).collect(),
			default_account: default_account.to_script_hash().unwrap(),
		})
	}
	pub fn save_to_file(&self, path: PathBuf) -> Result<(), WalletError> {
		// Convert wallet to NEP6
		let nep6 = self.to_nep6().unwrap();

		// Encode as JSON
		let json = serde_json::to_string(&nep6).unwrap();

		// Write to file at path
		let mut file = File::create(path).unwrap();
		file.write_all(json.as_bytes()).unwrap();

		Ok(())
	}

	pub fn get_account(&self, script_hash: &H160) -> Option<&Account> {
		self.accounts.get(script_hash)
	}

	pub fn remove_account(&mut self, script_hash: &H160) -> bool {
		self.accounts.remove(script_hash).is_some()
	}

	pub fn encrypt_accounts(&mut self, password: &str) {
		for account in self.accounts.values_mut() {
			account.encrypt_private_key(password);
		}
	}
}
