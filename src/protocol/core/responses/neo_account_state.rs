use p256::PublicKey;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct NeoAccountState {
    pub balance: i64,
    pub balance_height: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<PublicKey>,
}

impl NeoAccountState {

    pub fn with_no_vote(balance: i64, update_height: i64) -> Self {
        Self {
            balance,
            balance_height: Some(update_height),
            public_key: None,
        }
    }

    pub fn with_no_balance() -> Self {
        Self {
            balance: 0,
            balance_height: None,
            public_key: None,
        }
    }
}