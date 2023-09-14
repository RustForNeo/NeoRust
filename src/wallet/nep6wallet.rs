use std::collections::HashMap;
use crypto::scrypt::ScryptParams;
use serde::{Deserialize, Serialize};
use crate::wallet::nep6account::NEP6Account;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NEP6Wallet {
    pub name: String,
    pub version: String,
    pub scrypt: ScryptParams,
    pub accounts: Vec<NEP6Account>,
    pub extra: Option<HashMap<String, String>>
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