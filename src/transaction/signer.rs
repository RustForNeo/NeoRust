use std::hash::{Hash, Hasher};
use std::io::Write;
use bincode::Options;
use p256::pkcs8::der::Encode;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use crate::constant::NeoConstants;
use crate::neo_error::NeoError;
use crate::protocol::core::witness_rule::witness_condition::WitnessCondition;
use crate::protocol::core::witness_rule::witness_rule::WitnessRule;
use crate::serialization::binary_reader::BinaryReader;
use crate::serialization::binary_writer::BinaryWriter;
use crate::transaction::transaction_error::TransactionError;
use crate::transaction::witness_scope::WitnessScope;
use crate::types::ECPublicKey;

// #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
// pub struct Signer {
//     pub signer_hash: H160,
//     scopes: Vec<WitnessScope>,
//     allowed_contracts: Vec<H160>,
//     allowed_groups: Vec<ECPublicKey>,
//     rules: Vec<WitnessRule>,
// }

pub trait Signer: Clone+PartialEq+Serialize+Deserialize {

    fn get_signer_hash(&self) -> &H160;

    fn set_signer_hash(&mut self, signer_hash: H160);

    fn get_scopes(&self) -> &Vec<WitnessScope>;

    fn set_scopes(&mut self, scopes: Vec<WitnessScope>);

    fn get_allowed_contracts(&self) -> &Vec<H160>;

    // fn set_allowed_contracts(&mut self, allowed_contracts: Vec<H160>);

    // fn new(signer_hash: H160, scope: WitnessScope) -> Self {
    //     Self {
    //         signer_hash,
    //         scopes: vec![scope],
    //         allowed_contracts: Vec::new(),
    //         allowed_groups: Vec::new(),
    //         rules: Vec::new(),
    //     }
    // }

    // Setters

    // Set allowed contracts
    fn set_allowed_contracts(&mut self, contracts: Vec<H160>) -> Result<(), NeoError> {

        // Validate
        if self.get_scopes().contains(&WitnessScope::Global) {
            return Err(NeoError::InvalidConfiguration("Cannot set contracts for global scope".to_string()));
        }

        if self.get_allowed_contracts().len() + contracts.len() > NeoConstants::MAX_SIGNER_SUBITEMS as usize {
            return Err(NeoError::InvalidConfiguration("Too many allowed contracts".to_string()));
        }

        // Update state
        if !self.get_scopes().contains(&WitnessScope::CustomContracts) {
            self.get_scopes().push(WitnessScope::CustomContracts);
        }

        self.get_allowed_contracts().extend(contracts);

        Ok(())
    }


    // Set allowed groups
    fn set_allowed_groups(&mut self, groups: Vec<ECPublicKey>) -> Result<(), NeoError> {

        if self.get_scopes().contains(&WitnessScope::Global) {
            return Err(NeoError::InvalidConfiguration(
                "Cannot set groups for global scope".to_string()
            ));
        }

        if self.get_allowed_groups().len() + groups.len() > NeoConstants::MAX_SIGNER_SUBITEMS as usize {
            return Err(NeoError::InvalidConfiguration(
                "Too many allowed groups".to_string()
            ));
        }

        if !self.get_scopes().contains(&WitnessScope::CustomGroups) {
            self.get_scopes().push(WitnessScope::CustomGroups);
        }

        self.get_allowed_groups().extend(groups);

        Ok(())
    }

    // Set rules
    fn set_rules(&mut self, rules: Vec<WitnessRule>) -> Result<(), NeoError> {

        if self.get_scopes().contains(&WitnessScope::Global) {
            return Err(NeoError::InvalidConfiguration(
                "Cannot set rules for global scope".to_string()
            ));
        }

        if self.get_rules().len() + rules.len() > NeoConstants::MAX_SIGNER_SUBITEMS as usize {
            return Err(NeoError::InvalidConfiguration(
                "Too many rules".to_string()
            ));
        }

        // Validate nesting depth
        for rule in &rules {
            Self::validate_depth(&rule.condition, NeoConstants::MAX_NESTING_DEPTH)?;
        }

        if !self.get_scopes().contains(&WitnessScope::WitnessRules) {
            self.get_scopes().push(WitnessScope::WitnessRules);
        }

        self.get_rules().extend(rules);

        Ok(())
    }

    // Check depth recursively
    fn validate_depth(rule: &WitnessCondition, depth: u8) -> Result<(), NeoError> {

        // Depth exceeded
        if depth == 0 {
            return Err(NeoError::InvalidConfiguration(
                "Max nesting depth exceeded".to_string()
            ));
        }

        match &rule {
            WitnessCondition::And(conditions) |
            WitnessCondition::Or(conditions) => {
                for inner_rule in conditions {
                    Self::validate_depth(inner_rule, depth - 1)?;
                }
            },
            _ => ()
        }

        Ok(())
    }
    fn validate_subitems(count: usize, name: &str) -> Result<(), NeoError> {
        if count > NeoConstants::MAX_SIGNER_SUBITEMS as usize {
            return Err(NeoError::InvalidData(format!(
                "Too many {} in signer", name)));
        }
        Ok(())
    }
}

impl Hash for Signer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.signer_hash.hash(state);
        self.scopes.hash(state);
        self.allowed_contracts.hash(state);
        self.allowed_groups.hash(state);
        self.rules.hash(state);
    }
}

impl Serialize for Signer {
    fn serialize(&self, writer: &mut BinaryWriter) {

        // Write signer hash
        writer.write(&self.signer_hash.to_vec()).expect("Failed to write signer hash");

        // Combine scopes and write byte
        let combined_scopes = WitnessScope::combine(&self.scopes);
        writer.write_u8(combined_scopes);

        // Write allowed contracts if needed
        if self.scopes.contains(&WitnessScope::CustomContracts) {
            writer.write_serializable_list(&self.allowed_contracts);
        }

        // Write allowed groups if needed
        if self.scopes.contains(&WitnessScope::CustomGroups) {
            writer.write_serializable_list(&self.allowed_groups);
        }

        // Write rules if needed
        if self.scopes.contains(&WitnessScope::WitnessRules) {
            writer.write_serializable_list(&self.rules);
        }

    }
    fn serialized_size(&self) -> usize {
        let mut size = 0;
        size += self.signer_hash.to_vec().len();
        size += 1; // scopes
        if self.scopes.contains(&WitnessScope::CustomContracts) {
            size += self.allowed_contracts.serialized_size();
        }
        if self.scopes.contains(&WitnessScope::CustomGroups) {
            size += self.allowed_groups.serialized_size();
        }
        if self.scopes.contains(&WitnessScope::WitnessRules) {
            size += self.rules.serialized_size();
        }
        size
    }
}


impl Deserialize for Signer{
    fn deserialize(reader: &mut BinaryReader) -> Result<Self, TransactionError> {
        // Read signer hash
        let signer_hash = reader.read_serializable()?;

        // Read scopes
        let combined_scopes = reader.read_u8()?;
        let scopes = WitnessScope::extract(&combined_scopes);

        // Read other fields if scope requires it
        let mut allowed_contracts = Vec::new();
        if scopes.contains(&WitnessScope::CustomContracts) {
            allowed_contracts = reader.read_serializable_list()?;
            Self::validate_subitems(allowed_contracts.len(), "contracts")?;
        }

        let mut allowed_groups = Vec::new();
        if scopes.contains(&WitnessScope::CustomGroups) {
            allowed_groups = reader.read_serializable_list()?;
            Self::validate_subitems(allowed_groups.len(), "groups")?;
        }

        let mut rules = Vec::new();
        if scopes.contains(&WitnessScope::WitnessRules) {
            rules = reader.read_serializable_list()?;
            Self::validate_subitems(rules.len(), "rules")?;
        }

        Ok(Self {
            signer_hash,
            scopes,
            allowed_contracts,
            allowed_groups,
            rules,
        })
    }
}
