use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NeoGetUnclaimedGas {
	pub unclaimed_gas: Option<GetUnclaimedGas>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct GetUnclaimedGas {
	pub unclaimed: String,
	pub address: String,
}
