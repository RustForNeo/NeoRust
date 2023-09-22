use crate::types::{
	contract_parameter::ContractParameter, contract_parameter_type::ContractParameterType,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContractManifest {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(default)]
	pub groups: Vec<ContractGroup>,
	#[serde(skip_serializing)]
	pub features: Option<HashMap<String, serde_json::Value>>,
	#[serde(default)]
	#[serde(serialize_with = "serialize_wildcard")]
	#[serde(deserialize_with = "deserialize_wildcard")]
	pub supported_standards: Vec<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub abi: Option<ContractABI>,
	#[serde(default)]
	pub permissions: Vec<ContractPermission>,
	#[serde(skip_serializing)]
	#[serde(serialize_with = "serialize_wildcard")]
	#[serde(deserialize_with = "deserialize_wildcard")]
	pub trusts: Vec<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub extra: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContractGroup {
	pub pub_key: String,
	pub signature: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ContractABI {
	pub methods: Vec<ContractMethod>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub events: Option<Vec<ContractEvent>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContractMethod {
	pub name: String,
	pub parameters: Vec<ContractParameter>,
	pub offset: usize,
	pub return_type: ContractParameterType,
	pub safe: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ContractEvent {
	pub name: String,
	pub parameters: Vec<ContractParameter>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContractPermission {
	pub contract: String,
	// #[serde(serialize_with = "serialize_wildcard")]
	// #[serde(deserialize_with = "deserialize_wildcard")]
	pub methods: Vec<String>,
}

fn serialize_wildcard<S>(value: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	if value == &vec!["*".to_string()] {
		serializer.serialize_str("*")
	} else {
		value.serialize(serializer)
	}
}

fn deserialize_wildcard<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(&deserializer).unwrap();
	if s == "*" {
		Ok(vec!["*".to_string()])
	} else {
		Vec::<String>::deserialize(deserializer)
	}
}
