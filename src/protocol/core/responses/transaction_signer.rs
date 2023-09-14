use serde::{Serialize, Deserialize};
use std::hash::{Hash, Hasher};
use crate::transaction::witness_scope::WitnessScope;
use crate::types::hash160::H160;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TransactionSigner {
    #[serde(rename = "account")]
    pub account: H160,

    #[serde(rename = "scopes")]
    pub scopes: Vec<WitnessScope>,

    #[serde(rename = "allowedcontracts")]
    pub allowed_contracts: Option<Vec<String>>,

    #[serde(rename = "allowedgroups")]
    pub allowed_groups: Option<Vec<String>>,

    #[serde(rename = "rules")]
    pub rules: Option<Vec<WitnessRule>>,
}

impl TransactionSigner {

    pub fn new(account: H160, scopes: Vec<WitnessScope>) -> Self {
        Self {
            account,
            scopes,
            allowed_contracts: None,
            allowed_groups: None,
            rules: None,
        }
    }

    pub fn new_full(
        account: H160,
        scopes: Vec<WitnessScope>,
        allowed_contracts: Vec<String>,
        allowed_groups: Vec<String>,
        rules: Vec<WitnessRule>
    ) -> Self {
        Self {
            account,
            scopes,
            allowed_contracts: Some(allowed_contracts),
            allowed_groups: Some(allowed_groups),
            rules: Some(rules),
        }
    }

}

// Manual hash implementation due to nested vector fields
impl Hash for TransactionSigner {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.account.hash(state);
        self.scopes.hash(state);
        if let Some(contracts) = &self.allowed_contracts {
            contracts.hash(state);
        }
        if let Some(groups) = &self.allowed_groups {
            groups.hash(state);
        }
        if let Some(rules) = &self.rules {
            rules.hash(state);
        }
    }
}