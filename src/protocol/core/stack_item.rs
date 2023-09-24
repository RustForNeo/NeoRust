use crate::{
	script::op_code::OpCode,
	types::{Address, PublicKey, PublicKeyExtension},
};
use primitive_types::{H160, H256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// | doesn't satisfy `StackItem: Hash`
// | doesn't satisfy `StackItem: std::cmp::Eq`

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StackItem {
	#[serde(rename = "Any")]
	Any,

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

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
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

	pub fn as_bool(&self) -> Option<bool> {
		match self {
			StackItem::Boolean { value } => Some(*value),
			StackItem::Integer { value } => Some(value != &0),
			_ => None,
		}
	}

	pub fn as_string(&self) -> Option<String> {
		match self {
			StackItem::ByteString { value } | StackItem::Buffer { value } =>
				hex::decode(value).ok().map(|bytes| String::from_utf8(bytes).ok()).unwrap(),
			StackItem::Integer { value } => Some(value.to_string()),
			StackItem::Boolean { value } => Some(value.to_string()),
			_ => None,
		}
	}

	pub fn to_string(&self) -> String {
		match self {
			StackItem::Any => format!("Any"),
			StackItem::Pointer { value: pointer } => format!("Pointer{{value={}}}", pointer),
			StackItem::Boolean { value: boolean } => format!("Boolean{{value={}}}", boolean),
			StackItem::Integer { value: integer } => format!("Integer{{value={}}}", integer),
			StackItem::ByteString { value: string } => format!("ByteString{{value={:?}}}", string),
			StackItem::Buffer { value: buffer } => format!("Buffer{{value={:?}}}", buffer),
			StackItem::Array { value: array } => {
				let values = array.iter().map(StackItem::to_string).collect::<Vec<_>>().join(", ");
				format!("Array{{value=[{}]}}", values)
			},
			StackItem::Struct { value: _struct } => {
				let values =
					_struct.iter().map(StackItem::to_string).collect::<Vec<_>>().join(", ");
				format!("Struct{{value=[{}]}}", values)
			},
			StackItem::Map { value: map_value } => {
				// Iterate over pairs of elements in the vector
				// (assuming the vector has an even number of elements)
				let entries = map_value
					.iter()
					.map(|(entry)| {
						format!("{} -> {}", entry.key.to_string(), entry.value.to_string())
					})
					.collect::<Vec<_>>()
					.join(", ");
				format!("Map{{{{{}}}}}", entries)
			},
			StackItem::InteropInterface { id, interface } => {
				format!("InteropInterface{{id={}, interface={}}}", id, interface)
			},
		}
	}

	pub fn as_bytes(&self) -> Option<Vec<u8>> {
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

	pub fn as_array(&self) -> Option<Vec<StackItem>> {
		match self {
			StackItem::Array { value } | StackItem::Struct { value } => Some(value.clone()),
			_ => None,
		}
	}

	pub fn as_int(&self) -> Option<i64> {
		match self {
			StackItem::Integer { value } => Some(*value),
			StackItem::Boolean { value } => Some(if *value { 1 } else { 0 }),
			_ => None,
		}
	}

	pub fn as_map(&self) -> Option<HashMap<StackItem, StackItem>> {
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

	pub fn as_address(&self) -> Option<Address> {
		self.as_bytes().and_then(|bytes| Some(Address::from_slice(&bytes)))
	}
	pub fn as_public_key(&self) -> Option<PublicKey> {
		self.as_bytes().and_then(|bytes| PublicKey::from_slice(&bytes).ok())
	}

	pub fn as_hash160(&self) -> Option<H160> {
		self.as_bytes().and_then(|bytes| Some(H160::from_slice(&bytes)))
	}

	pub fn as_hash256(&self) -> Option<H256> {
		self.as_bytes().and_then(|bytes| Some(H256::from_slice(&bytes)))
	}
	pub fn as_interop(&self, interface_name: &str) -> Option<StackItem> {
		match self {
			StackItem::Integer { value } => Some(StackItem::InteropInterface {
				id: value.to_string(),
				interface: interface_name.to_string(),
			}),
			StackItem::Boolean { value } => Some(StackItem::InteropInterface {
				id: value.to_string(),
				interface: interface_name.to_string(),
			}),
			StackItem::ByteString { value } => Some(StackItem::InteropInterface {
				id: value.to_string(),
				interface: interface_name.to_string(),
			}),
			StackItem::Buffer { value } => Some(StackItem::InteropInterface {
				id: value.to_string(),
				interface: interface_name.to_string(),
			}),
			_ => None,
		}
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

	pub fn get(&self, index: usize) -> Option<StackItem> {
		self.as_array().and_then(|arr| arr.get(index).cloned())
	}

	pub fn to_json(&self) -> Option<String> {
		serde_json::to_string(self).ok()
	}

	pub fn from_json(json: &str) -> Option<Self> {
		serde_json::from_str(json).ok()
	}

	// ...
}

impl From<String> for StackItem {
	fn from(value: String) -> Self {
		StackItem::ByteString { value }
	}
}

impl From<H160> for StackItem {
	fn from(value: H160) -> Self {
		StackItem::ByteString { value: value.to_string() }
	}
}

impl From<u8> for StackItem {
	fn from(value: u8) -> Self {
		StackItem::Integer { value: value as i64 }
	}
}

impl From<i8> for StackItem {
	fn from(value: i8) -> Self {
		StackItem::Integer { value: value as i64 }
	}
}

impl From<u16> for StackItem {
	fn from(value: u16) -> Self {
		StackItem::Integer { value: value as i64 }
	}
}

impl From<i16> for StackItem {
	fn from(value: i16) -> Self {
		StackItem::Integer { value: value as i64 }
	}
}

impl From<u32> for StackItem {
	fn from(value: u32) -> Self {
		StackItem::Integer { value: value as i64 }
	}
}

impl From<i32> for StackItem {
	fn from(value: i32) -> Self {
		StackItem::Integer { value: value as i64 }
	}
}

impl From<u64> for StackItem {
	fn from(value: u64) -> Self {
		StackItem::Integer { value: value as i64 }
	}
}
impl From<&str> for StackItem {
	fn from(value: &str) -> Self {
		StackItem::ByteString { value: value.to_string() }
	}
}
