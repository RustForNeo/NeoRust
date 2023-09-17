use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct NeoNetworkFee {
	#[serde(rename = "networkfee")]
	pub network_fee: u64,
}
