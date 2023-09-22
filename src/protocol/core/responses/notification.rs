use crate::{protocol::core::stack_item::StackItem, utils::*};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Notification {
	#[serde(deserialize_with = "deserialize_address")]
	#[serde(serialize_with = "serialize_address")]
	pub contract: H160,
	#[serde(rename = "eventname")]
	pub event_name: String,
	pub state: StackItem,
}

impl Notification {
	pub fn new(contract: H160, event_name: String, state: StackItem) -> Self {
		Self { contract, event_name, state }
	}
}
