use serde::{Deserialize, Serialize};
use crate::types::hash160::H160;

#[derive(Serialize, Deserialize, Hash)]
pub struct Diagnostics {
    pub invoked_contracts: InvokedContract,
    pub storage_changes: Vec<StorageChange>,
}

impl Diagnostics {
    pub fn new(invoked_contracts: InvokedContract, storage_changes: Vec<StorageChange>) -> Self {
        Self {
            invoked_contracts,
            storage_changes,
        }
    }
}

#[derive(Serialize, Deserialize, Hash)]
pub struct InvokedContract {
    pub hash: H160,
    pub invoked_contracts: Option<Vec<InvokedContract>>,
}

#[derive(Serialize, Deserialize, Hash)]
pub struct StorageChange {
    pub state: String,
    pub key: String,
    pub value: String,
}