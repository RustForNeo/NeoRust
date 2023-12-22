use crate::core::account::AccountTrait;
use neo_types::ScryptParamsDef;
use primitive_types::H160;
use std::collections::HashMap;

pub trait WalletTrait {
	type Account: AccountTrait;

	fn name(&self) -> &String;
	fn version(&self) -> &String;
	fn scrypt_params(&self) -> &ScryptParamsDef;
	fn accounts(&self) -> &HashMap<H160, Self::Account>;
	fn default_account(&self) -> &H160;
	fn set_name(&mut self, name: String);
	fn set_version(&mut self, version: String);
	fn set_scrypt_params(&mut self, params: ScryptParamsDef);
	fn set_default_account(&mut self, default_account: H160);
	fn add_account(&mut self, account: Self::Account);
	fn remove_account(&mut self, hash: &H160) -> Option<Self::Account>;
}
