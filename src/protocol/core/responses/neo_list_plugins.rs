use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct NeoListPlugins {
	pub plugins: Option<Vec<Plugin>>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Plugin {
	pub name: String,
	pub version: String,
	pub interfaces: Vec<String>,
}
