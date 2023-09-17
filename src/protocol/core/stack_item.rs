use crate::types::Address;
use primitive_types::{H160, H256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StackItem {
	#[serde(rename = "Any")]
	Any { value: Option<serde_json::Value> },

	#[serde(rename = "Pointer")]
	Pointer { value: i64 },

	#[serde(rename = "Boolean")]
	Boolean { value: bool },

	#[serde(rename = "Integer")]
	Integer { value: i64 },

	#[serde(rename = "ByteString")]
	ByteString {
		value: String, // hex encoded
	},

	#[serde(rename = "Buffer")]
	Buffer {
		value: String, // hex encoded
	},

	#[serde(rename = "Array")]
	Array { value: Vec<StackItem> },

	#[serde(rename = "Struct")]
	Struct { value: Vec<StackItem> },

	#[serde(rename = "Map")]
	Map { value: Vec<MapEntry> },

	#[serde(rename = "InteropInterface")]
	InteropInterface { id: String, interface: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapEntry {
	key: StackItem,
	value: StackItem,
}

// Utility methods

impl StackItem {
	pub const ANY_VALUE: &'static str = "Any";

	pub const POINTER_VALUE: &'static str = "Pointer";

	pub const BOOLEAN_VALUE: &'static str = "Boolean";

	pub const INTEGER_VALUE: &'static str = "Integer";

	pub const BYTE_STRING_VALUE: &'static str = "ByteString";

	pub const BUFFER_VALUE: &'static str = "Buffer";

	pub const ARRAY_VALUE: &'static str = "Array";

	pub const STRUCT_VALUE: &'static str = "Struct";

	pub const MAP_VALUE: &'static str = "Map";

	pub const INTEROP_INTERFACE_VALUE: &'static str = "InteropInterface";

	pub const ANY_BYTE: u8 = 0x00;

	pub const POINTER_BYTE: u8 = 0x10;

	pub const BOOLEAN_BYTE: u8 = 0x20;

	pub const INTEGER_BYTE: u8 = 0x21;

	pub const BYTE_STRING_BYTE: u8 = 0x28;

	pub const BUFFER_BYTE: u8 = 0x30;

	pub const ARRAY_BYTE: u8 = 0x40;

	pub const STRUCT_BYTE: u8 = 0x41;

	pub const MAP_BYTE: u8 = 0x48;

	pub const INTEROP_INTERFACE_BYTE: u8 = 0x60;

	fn as_bool(&self) -> Option<bool> {
		match self {
			StackItem::Boolean { value } => Some(*value),
			StackItem::Integer { value } => Some(value != &0),
			_ => None,
		}
	}

	fn as_string(&self) -> Option<String> {
		match self {
			StackItem::ByteString { value } | StackItem::Buffer { value } =>
				hex::decode(value).ok().map(|bytes| String::from_utf8(bytes).ok())?,
			StackItem::Integer { value } => Some(value.to_string()),
			StackItem::Boolean { value } => Some(value.to_string()),
			_ => None,
		}
	}

	pub fn to_string(&self) -> String {
		match self {
			StackItem::Any(value) => format!("Any{{value={:?}}}", value),
			StackItem::Pointer(value) => format!("Pointer{{value={}}}", value),
			StackItem::Boolean(value) => format!("Boolean{{value={}}}", value),
			StackItem::Integer(value) => format!("Integer{{value={}}}", value),
			StackItem::ByteString(value) => format!("ByteString{{value={:?}}}", value),
			StackItem::Buffer(value) => format!("Buffer{{value={:?}}}", value),
			StackItem::Array(value) => {
				let values = value.iter().map(StackItem::to_string).collect::<Vec<_>>().join(", ");
				format!("Array{{value=[{}]}}", values)
			},
			StackItem::Struct(value) => {
				let values = value.iter().map(StackItem::to_string).collect::<Vec<_>>().join(", ");
				format!("Struct{{value=[{}]}}", values)
			},
			StackItem::Map(value) => {
				let entries = value
					.iter()
					.map(|(k, v)| format!("{} -> {}", k.to_string(), v.to_string()))
					.collect::<Vec<_>>()
					.join(", ");
				format!("Map{{{{{}}}}}", entries)
			},
			StackItem::InteropInterface(id, interface) => {
				format!("InteropInterface{{id={}, interface={}}}", id, interface)
			},
		}
	}

	fn as_bytes(&self) -> Option<Vec<u8>> {
		match self {
			StackItem::ByteString { value } | StackItem::Buffer { value } =>
				hex::decode(value).ok(),
			StackItem::Integer { value } => {
				let mut bytes = value.to_be_bytes().to_vec();
				bytes.reverse();
				Some(bytes)
			},
			_ => None,
		}
	}

	fn as_array(&self) -> Option<Vec<StackItem>> {
		match self {
			StackItem::Array { value } | StackItem::Struct { value } => Some(value.clone()),
			_ => None,
		}
	}

	fn as_int(&self) -> Option<i64> {
		match self {
			StackItem::Integer { value } => Some(*value),
			StackItem::Boolean { value } => Some(if *value { 1 } else { 0 }),
			_ => None,
		}
	}

	fn as_map(&self) -> Option<HashMap<StackItem, StackItem>> {
		match self {
			StackItem::Map { value } => {
				let mut map = HashMap::new();
				for entry in value {
					map.insert(entry.key.clone(), entry.value.clone());
				}
				Some(map)
			},
			_ => None,
		}
	}

	fn as_address(&self) -> Option<Address> {
		self.as_bytes().and_then(|bytes| Address::from_bytes(&bytes).ok())
	}

	fn as_hash160(&self) -> Option<H160> {
		self.as_bytes().and_then(|bytes| H160::from_bytes(&bytes).ok())
	}

	fn as_hash256(&self) -> Option<H256> {
		self.as_bytes().and_then(|bytes| H256::from_bytes(&bytes).ok())
	}
	pub fn len(&self) -> Option<usize> {
		match self {
			StackItem::Array { value } | StackItem::Struct { value } => Some(value.len()),
			_ => None,
		}
	}

	pub fn is_empty(&self) -> Option<bool> {
		self.len().map(|len| len == 0)
	}

	pub fn get(&self, index: usize) -> Option<&StackItem> {
		self.as_array().and_then(|arr| arr.get(index))
	}

	pub fn to_json(&self) -> Option<String> {
		serde_json::to_string(self).ok()
	}

	pub fn from_json(json: &str) -> Option<Self> {
		serde_json::from_str(json).ok()
	}

	// ...
}
