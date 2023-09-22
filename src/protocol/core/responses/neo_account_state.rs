use crate::{types::PublicKey, utils::*};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct AccountState {
	pub balance: i64,
	pub balance_height: Option<i64>,
	#[serde(deserialize_with = "deserialize_public_key_option")]
	#[serde(serialize_with = "serialize_public_key_option")]
	pub public_key: Option<PublicKey>,
}

impl AccountState {
	pub fn with_no_vote(balance: i64, update_height: i64) -> Self {
		Self { balance, balance_height: Some(update_height), public_key: None }
	}

	pub fn with_no_balance() -> Self {
		Self { balance: 0, balance_height: None, public_key: None }
	}
}
