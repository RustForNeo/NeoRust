use serde::{Serialize, Deserialize};
use base64::{encode, decode};
use crypto::sha3::Sha3Mode::Keccak256;
use serde_json::json;
use sha3::{Digest};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ContractParameterType {
    Any,
    Boolean,
    Integer,
    ByteArray,
    String,
    Hash160,
    Hash256,
    PublicKey,
    Signature,
    Array,
    Map,
}

impl ContractParameterType {
    fn as_str(&self) -> &str {
        match self {
            Self::Any => "Any",
            Self::Boolean => "Boolean",
            Self::Integer => "Integer",
            Self::ByteArray => "ByteArray",
            Self::String => "String",
            Self::Hash160 => "Hash160",
            Self::Hash256 => "Hash256",
            Self::PublicKey => "PublicKey",
            Self::Signature => "Signature",
            Self::Array => "Array",
            Self::Map => "Map",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractParameter {
    name: Option<String>,
    #[serde(rename = "type")]
    typ: ContractParameterType,
    pub(crate) value: Option<ParameterValue>,
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParameterValue {
    Boolean(bool),
    Integer(i64),
    ByteArray(String),
    String(String),
    Hash160(String),
    Hash256(String),
    PublicKey(String),
    Signature(String),
    Array(Vec<ContractParameter>),
    Map(Vec<serde_json::Value>),
}

impl ContractParameter {
    pub fn new(typ: ContractParameterType) -> Self {
        Self {
            name: None,
            typ,
            value: None,
        }
    }

    pub fn with_value(typ: ContractParameterType, value: ParameterValue) -> Self {
        Self {
            name: None,
            typ,
            value: Some(value),
        }
    }

    pub fn bool(value: bool) -> Self {
        Self::with_value(ContractParameterType::Boolean, ParameterValue::Boolean(value))
    }

    pub fn integer(value: i64) -> Self {
        Self::with_value(ContractParameterType::Integer, ParameterValue::Integer(value))
    }

    pub fn byte_array(value: Vec<u8>) -> Self {
        let encoded = encode(value);
        Self::with_value(ContractParameterType::ByteArray, ParameterValue::ByteArray(encoded))
    }

    pub fn string(value: String) -> Self {
        Self::with_value(ContractParameterType::String, ParameterValue::String(value))
    }

    // Other helper methods
    pub fn hash160(value: &str) -> Self {
        Self::with_value(ContractParameterType::Hash160, ParameterValue::Hash160(value.to_string()))
    }

    pub fn hash256(value: &str) -> Self {
        Self::with_value(ContractParameterType::Hash256, ParameterValue::Hash256(value.to_string()))
    }

    pub fn public_key(value: &str) -> Self {
        Self::with_value(ContractParameterType::PublicKey, ParameterValue::PublicKey(value.to_string()))
    }

    pub fn signature(value: &str) -> Self {
        Self::with_value(ContractParameterType::Signature, ParameterValue::Signature(value.to_string()))
    }

    pub fn array(values: Vec<Self>) -> Self {
        Self::with_value(ContractParameterType::Array, ParameterValue::Array(values))
    }

    pub fn map(values: Vec<(Self, Self)>) -> Self {
        let json = values
            .into_iter()
            .map(|(k, v)| json!({"key": k, "value": v}))
            .collect();

        Self::with_value(ContractParameterType::Map, ParameterValue::Map(json))
    }
    pub fn hash(self) -> Vec<u8> {
        let mut hasher = Keccak256::new();
        hasher.update(self.name.unwrap_or_default());
        hasher.update(self.typ.as_str());
        hasher.update(self.value.unwrap_or_default());
        hasher.finalize().to_vec()
    }
}