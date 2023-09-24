use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct NeoGetWalletBalance {
	pub wallet_balance: Option<Balance>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Balance {
	#[serde(alias = "Balance")]
	pub balance: String,
}
