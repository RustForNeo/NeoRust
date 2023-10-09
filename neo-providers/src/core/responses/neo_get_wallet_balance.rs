use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Balance {
	#[serde(alias = "Balance")]
	pub balance: String,
}
