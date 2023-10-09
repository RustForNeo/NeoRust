use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Plugin {
	pub name: String,
	pub version: String,
	pub interfaces: Vec<String>,
}
