use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct NeoGetWalletBalance {
    pub wallet_balance: Option<Balance>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Balance {
    #[serde(alias = "Balance")]
    pub balance: String,
}