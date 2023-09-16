use primitive_types::H160;
use serde::{Deserialize, Serialize};
use crate::protocol::core::witness_rule::witness_rule::WitnessRule;
use crate::transaction::signer::Signer;
use crate::transaction::witness_scope::WitnessScope;
use crate::types::contract_parameter::ContractParameter;
use crate::types::ECPublicKey;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct ContractSigner {
    signer_hash: H160,
    scopes: Vec<WitnessScope>,
    allowed_contracts: Vec<H160>,
    allowed_groups: Vec<ECPublicKey>,
    rules: Vec<WitnessRule>,
    pub verify_params: Vec<ContractParameter>,
    contract_hash: H160,
    scope: WitnessScope
}

impl Signer for ContractSigner{
    type SignerType = ContractSigner;

    fn get_signer_hash(&self) -> &H160 {
        &self.signer_hash
    }

    fn set_signer_hash(&mut self, signer_hash: H160) {
        self.signer_hash = signer_hash;
    }

    fn get_scopes(&self) -> &Vec<WitnessScope> {
        &self.scopes
    }

    fn set_scopes(&mut self, scopes: Vec<WitnessScope>) {
        self.scopes = scopes;
    }

    fn get_allowed_contracts(&self) -> &Vec<H160> {
        &self.allowed_contracts
    }
}

impl ContractSigner {

    fn new(contract_hash: H160, scope: WitnessScope, verify_params: Vec<ContractParameter>) -> Self {
        Self {
            signer_hash: Default::default(),
            scopes: vec![],
            allowed_contracts: vec![],
            allowed_groups: vec![],
            rules: vec![],
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