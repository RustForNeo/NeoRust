use std::{collections::HashMap, path::PathBuf};
use std::fs::File;
use std::io::Write;
use primitive_types::H160;

use crate::crypto::scrypt_parameter::ScryptParams;
use crate::wallet::account::Account;
use crate::wallet::nep6wallet::NEP6Wallet;
use crate::wallet::wallet_error::WalletError;

pub struct Wallet {
    name: String,
    version: String,
    scrypt_params: ScryptParams,

    accounts: HashMap<H160, Account>,
    default_account: H160,
}

impl Wallet {

    // Constructor

    pub fn new() -> Self {
        Self {
            name: "MyWallet".to_string(),
            version: "1.0".to_string(),
            scrypt_params: ScryptParams::default(),
            accounts: HashMap::new(),
            default_account: H160::default(),
        }
    }

    // Configuration methods

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn add_account(&mut self, account: Account) {
        self.accounts.insert(account.get_script_hash(), account);
    }

    pub fn set_default_account(&mut self, script_hash: H160) {
        self.default_account = script_hash;
    }

    // Serialization methods

    pub fn to_nep6(&self) -> Result<NEP6Wallet, WalletError> {
        let accounts = self.accounts.values()
            .map(|a| a.to_nep6())
            .collect();

        Ok(NEP6Wallet {
            name: self.name.clone(),
            version: self.version.clone(),
            scrypt: self.scrypt_params.clone(),
            accounts,
            extra: None,
        })
    }

    pub fn from_nep6(nep6: NEP6Wallet) -> Result<Self, WalletError> {
        let accounts = nep6.accounts
            .into_iter()
            .map(Account::from_nep6)
            .collect();

        let default_account = nep6.accounts
            .iter()
            .find(|a| a.is_default)
            .map(|a| a.get_script_hash())
            .ok_or(WalletError::NoDefaultAccount)?;

        Ok(Self {
            name: nep6.name,
            version: nep6.version,
            scrypt_params: nep6.scrypt,
            accounts,
            default_account,
        })
    }
    pub fn save_to_file(&self, path: PathBuf) -> Result<(), WalletError> {

        // Convert wallet to NEP6
        let nep6 = self.to_nep6()?;

        // Encode as JSON
        let json = serde_json::to_string(&nep6)?;

        // Write to file at path
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;

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