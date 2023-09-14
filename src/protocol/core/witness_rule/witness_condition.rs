use std::io;
use std::io::Write;
use primitive_types::H160;
use serde::{Deserialize, Deserializer, Serialize};
use crate::types::ECPublicKey;

#[derive(Hash, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WitnessCondition {
    Boolean(bool),
    Not(Box<WitnessCondition>),
    And(Vec<WitnessCondition>),
    Or(Vec<WitnessCondition>),
    ScriptHash(H160),
    Group(ECPublicKey),
    CalledByEntry,
    CalledByContract(H160),
    CalledByGroup(ECPublicKey),
}

impl WitnessCondition {
    const MAX_SUBITEMS: usize = 16;
    const MAX_NESTING_DEPTH: usize = 2;

    const BOOLEAN_VALUE: &'static str = "Boolean";
    const NOT_VALUE: &'static str = "Not";
    const AND_VALUE: &'static str = "And";
    const OR_VALUE: &'static str = "Or";
    const SCRIPT_HASH_VALUE: &'static str = "ScriptHash";
    const GROUP_VALUE: &'static str = "Group";
    const CALLED_BY_ENTRY_VALUE: &'static str = "CalledByEntry";
    const CALLED_BY_CONTRACT_VALUE: &'static str = "CalledByContract";
    const CALLED_BY_GROUP_VALUE: &'static str = "CalledByGroup";

    const BOOLEAN_BYTE: u8 = 0x00;
    const NOT_BYTE: u8 = 0x01;
    const AND_BYTE: u8 = 0x02;
    const OR_BYTE: u8 = 0x03;
    const SCRIPT_HASH_BYTE: u8 = 0x18;
    const GROUP_BYTE: u8 = 0x19;
    const CALLED_BY_ENTRY_BYTE: u8 = 0x20;
    const CALLED_BY_CONTRACT_BYTE: u8 = 0x28;
    const CALLED_BY_GROUP_BYTE: u8 = 0x29;

    pub fn json_value(&self) -> &'static str {
        match self {
            WitnessCondition::Boolean(_) => WitnessCondition::BOOLEAN_VALUE,
            WitnessCondition::Not(_) => WitnessCondition::NOT_VALUE,
            WitnessCondition::And(_) => WitnessCondition::AND_VALUE,
            WitnessCondition::Or(_) => WitnessCondition::OR_VALUE,
            WitnessCondition::ScriptHash(_) => WitnessCondition::SCRIPT_HASH_VALUE,
            WitnessCondition::Group(_) => WitnessCondition::GROUP_VALUE,
            WitnessCondition::CalledByEntry => WitnessCondition::CALLED_BY_ENTRY_VALUE,
            WitnessCondition::CalledByContract(_) => WitnessCondition::CALLED_BY_CONTRACT_VALUE,
            WitnessCondition::CalledByGroup(_) => WitnessCondition::CALLED_BY_GROUP_VALUE,
        }
    }

    pub fn byte(&self) -> u8 {
        match self {
            WitnessCondition::Boolean(_) => WitnessCondition::BOOLEAN_BYTE,
            WitnessCondition::Not(_) => WitnessCondition::NOT_BYTE,
            WitnessCondition::And(_) => WitnessCondition::AND_BYTE,
            WitnessCondition::Or(_) => WitnessCondition::OR_BYTE,
            WitnessCondition::ScriptHash(_) => WitnessCondition::SCRIPT_HASH_BYTE,
            WitnessCondition::Group(_) => WitnessCondition::GROUP_BYTE,
            WitnessCondition::CalledByEntry => WitnessCondition::CALLED_BY_ENTRY_BYTE,
            WitnessCondition::CalledByContract(_) => WitnessCondition::CALLED_BY_CONTRACT_BYTE,
            WitnessCondition::CalledByGroup(_) => WitnessCondition::CALLED_BY_GROUP_BYTE,
        }
    }

    // other methods

    pub fn boolean_expression(&self) -> Option<bool> {
        match self {
            WitnessCondition::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn expression(&self) -> Option<&WitnessCondition> {
        match self {
            WitnessCondition::Not(exp) => Some(&exp),
            _ => None,
        }
    }

    pub fn expression_list(&self) -> Option<&[WitnessCondition]> {
        match self {
            WitnessCondition::And(exp) | WitnessCondition::Or(exp) => Some(&exp),
            _ => None,
        }
    }

    pub fn script_hash(&self) -> Option<&H160> {
        match self {
            WitnessCondition::ScriptHash(hash) | WitnessCondition::CalledByContract(hash) => Some(hash),
            _ => None,
        }
    }

    pub fn group(&self) -> Option<&ECPublicKey> {
        match self {
            WitnessCondition::Group(group) | WitnessCondition::CalledByGroup(group) => Some(group),
            _ => None,
        }
    }

}
// Serialization

impl Serialize for WitnessCondition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
    {
        use serde::ser::SerializeTuple;
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&self.json_value())?;
        match self {
            WitnessCondition::Boolean(b) => tuple.serialize_element(b),
            WitnessCondition::Not(exp) => {
                tuple.serialize_element(&exp.serialize(serializer)?)
            }
            WitnessCondition::And(exps) | WitnessCondition::Or(exps) => {
                tuple.serialize_element(&exps.serialize(serializer)?)
            }
            WitnessCondition::ScriptHash(hash) | WitnessCondition::CalledByContract(hash) => {
                tuple.serialize_element(&hash.serialize(serializer)?)
            }
            WitnessCondition::Group(group) | WitnessCondition::CalledByGroup(group) => {
                tuple.serialize_element(&group.serialize(serializer)?)
            }
            WitnessCondition::CalledByEntry => {}
        }
        tuple.end()
    }
}

// Deserialization

impl<'de> Deserialize<'de> for WitnessCondition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let (type_str, value) = Deserialize::deserialize(deserializer)?;

        let condition = match type_str.as_str() {
            WitnessCondition::BOOLEAN_VALUE => {
                WitnessCondition::Boolean(value.unwrap())
            }
            WitnessCondition::NOT_VALUE => {
                WitnessCondition::Not(Box::new(value.unwrap()))
            }
            WitnessCondition::AND_VALUE | WitnessCondition::OR_VALUE => {
                let exp_vec = value.unwrap();
                if type_str == WitnessCondition::AND_VALUE {
                    WitnessCondition::And(exp_vec)
                } else {
                    WitnessCondition::Or(exp_vec)
                }
            }
            WitnessCondition::SCRIPT_HASH_VALUE | WitnessCondition::CALLED_BY_CONTRACT_VALUE => {
                WitnessCondition::ScriptHash(value.unwrap())
            }
            WitnessCondition::GROUP_VALUE | WitnessCondition::CALLED_BY_GROUP_VALUE => {
                WitnessCondition::Group(value.unwrap())
            }
            WitnessCondition::CALLED_BY_ENTRY_VALUE => {
                WitnessCondition::CalledByEntry
            }
            _ => {
                return Err(Error::custom("invalid type"));
            }
        };

        Ok(condition)
    }
}