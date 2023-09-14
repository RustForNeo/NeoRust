use serde::{Deserialize, Serialize};
use crate::transaction::witness_scope::WitnessScope;
use crate::types::contract_parameter::ContractParameter;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct ContractSigner {
    pub verify_params: Vec<ContractParameter>,
    contract_hash: H160,
    scope: WitnessScope
}

impl ContractSigner {

    fn new(contract_hash: H160, scope: WitnessScope, verify_params: Vec<ContractParameter>) -> Self {
        Self {
            verify_params,
            contract_hash,
            scope
        }
    }

    pub fn called_by_entry(contract_hash: H160, verify_params: &[ContractParameter]) -> Self {
        Self::new(contract_hash, WitnessScope::CalledByEntry, verify_params.to_vec())
    }

    pub fn global(contract_hash: H160, verify_params: &[ContractParameter]) -> Self {
        Self::new(contract_hash, WitnessScope::Global, verify_params.to_vec())
    }
}