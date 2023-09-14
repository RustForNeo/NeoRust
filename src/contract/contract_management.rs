// contract_management

use primitive_types::H160;
use serde::{Deserialize, Serialize};
use crate::contract::contract_error::ContractError;
use crate::contract::nef_file::NefFile;
use crate::protocol::core::responses::contract_state::{ContractIdentifiers, ContractState};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::types::contract_parameter::ContractParameter;

#[derive(Debug, Clone, PartialEq, Eq,Serialize, Deserialize)]
pub struct ContractManagement {
    script_hash: H160,
}

impl ContractManagement {
    pub fn new(script_hash: H160) -> Self {
        Self {
            script_hash,
        }
    }

    pub async fn get_minimum_deployment_fee(&self) -> Result<u64, Err> {
        self.client.call_function(
            self.script_hash.clone(),
            "getMinimumDeploymentFee",
            vec![]
        ).await
    }

    pub async fn set_minimum_deployment_fee(&self, fee: u64) -> Result<u64, Err> {
        self.client.call_function(
            self.script_hash.clone(),
            "setMinimumDeploymentFee",
            vec![fee.into()]
        ).await
    }

    pub async fn get_contract(&self, hash: H160) -> Result<ContractState, ContractError> {
        self.client.get_contract(hash).await
    }

    pub async fn get_contract_by_id(&self, id: u32) -> Result<ContractState, ContractError> {
        let hash = self.get_contract_hash_by_id(id).await?;
        self.get_contract(hash).await
    }

    pub async fn get_contract_hash_by_id(&self, id: u32) -> Result<H160, ContractError> {
        let result = self.client.call_function(
            self.script_hash.clone(),
            "getContractById",
            vec![id.into()]
        ).await?;

        let item = &result[0];
        Ok(H160::from(item.as_bytes()?))
    }

    pub async fn get_contract_hashes(&self) -> Result<ContractIdentifiers, ContractError> {
        self.client
            .call_function_iter(self.script_hash.clone, "getContractHashes", vec![])
            .await
            .map(|item| ContractIdentifiers::try_from(item))
    }

    pub async fn has_method(&self, hash: H160, method: &str, params: usize) -> Result<bool, ContractError> {
        self.client.call_function(
            self.script_hash.clone(),
            "hasMethod",
            vec![hash.into(), method.into(), params.into()]
        ).await
    }

    pub async fn deploy(&self, nef: &NefFile, manifest: &[u8], data: Option<ContractParameter>) -> Result<TransactionBuilder, Err> {
        let params = vec![nef.into(), manifest.into(), data];
        let tx = TransactionBuilder::call_function(self.script_hash.clone(), "deploy", params);
        Ok(tx)
    }

}

// Other types and helpers