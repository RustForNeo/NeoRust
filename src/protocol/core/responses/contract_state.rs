use primitive_types::H160;
use serde::{Deserialize, Serialize};
use crate::protocol::core::responses::contract_manifest::ContractManifest;
use crate::protocol::core::responses::contract_nef::ContractNef;
use crate::protocol::core::stack_item::StackItem;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractState {
    pub id: i32,
    pub nef: ContractNef,
    pub update_counter: i32,
    pub hash: H160,
    pub manifest: ContractManifest,
}

impl ContractState {

    pub fn new(id: i32, update_counter: i32, hash: H160, nef: ContractNef, manifest: ContractManifest) -> Self {
        Self {
            id,
            nef,
            update_counter,
            hash,
            manifest,
        }
    }

    pub fn contract_identifiers(stack_item: &StackItem) -> Result<ContractIdentifiers, &'static str> {
        match stack_item {
            StackItem::Struct(values) if values.len() >= 2 => {
                let id = values[0].to_i32()?;
                let hash = H160::from_slice(&values[1].to_array()?.as_slice().reverse())?;
                Ok(ContractIdentifiers { id, hash })
            }
            _ => Err("Could not deserialize ContractIdentifiers from stack item")
        }
    }

}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ContractIdentifiers {
    pub id: i32,
    pub hash: H160
}