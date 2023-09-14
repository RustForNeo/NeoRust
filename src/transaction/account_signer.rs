use primitive_types::H160;
use serde::{Deserialize, Serialize};
use crate::transaction::transaction_error::TransactionError;
use crate::transaction::witness_scope::WitnessScope;
use crate::wallet::account::Account;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct AccountSigner {
    pub account: Account,
    scope: WitnessScope,
}

impl AccountSigner {

    fn new(account: Account, scope: WitnessScope) -> Self {
        Self {
            account,
            scope,
        }
    }

    pub fn none(account: Account) -> Result<Self, TransactionError> {
        Ok(Self::new(account, WitnessScope::None))
    }

    pub fn none_hash160(account_hash: H160) -> Result<Self, TransactionError> {
        let account = Account::from_address(account_hash.to_address());
        Ok(Self::new(account, WitnessScope::None))
    }

    pub fn called_by_entry(account: Account) -> Result<Self, TransactionError> {
        Ok(Self::new(account, WitnessScope::CalledByEntry))
    }

    pub fn called_by_entry_hash160(account_hash: H160) -> Result<Self, TransactionError> {
        let account = Account::from_address(account_hash.to_address());
        Ok(Self::new(account, WitnessScope::CalledByEntry))
    }

    pub fn global(account: Account) -> Result<Self, TransactionError> {
        Ok(Self::new(account, WitnessScope::Global))
    }

    pub fn global_hash160(account_hash: H160) -> Result<Self, TransactionError> {
        let account = Account::from_address(account_hash.to_address());
        Ok(Self::new(account, WitnessScope::Global))
    }

}