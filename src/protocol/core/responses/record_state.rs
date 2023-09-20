use crate::protocol::core::stack_item::StackItem;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RecordState {
	pub name: String,
	pub record_type: RecordType,
	pub data: String,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RecordType {
	A = 0x01,
	AAAA = 0x02,
	CNAME = 0x04,
	Delete = 0x08,
}

impl RecordState {
	pub fn new(name: String, record_type: RecordType, data: String) -> Self {
		Self { name, record_type, data }
	}

	pub fn from_stack_item(item: &StackItem) -> Result<Self, &'static str> {
		match item {
			StackItem::Array(vec) if vec.len() == 3 => {
				if let Some(name) = vec[0].as_str() {
					if let Some(byte) = vec[1].as_i8() {
						if let Some(record_type) = RecordType::from_u8(byte) {
							if let Some(data) = vec[2].as_str() {
								return Ok(Self::new(name, record_type, data))
							}
						}
					}
				}
				Err("Could not deserialize RecordState")
			},
			_ => Err("Expected a StackItem array of length 3"),
		}
	}
}
