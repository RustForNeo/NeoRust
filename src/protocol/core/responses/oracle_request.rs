use primitive_types::{H160, H256};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OracleRequest {
	#[serde(rename = "requestid")]
	pub request_id: i32,

	#[serde(rename = "originaltxid")]
	pub original_transaction_hash: H256,

	#[serde(rename = "gasforresponse")]
	pub gas_for_response: i32,

	pub url: String,

	pub filter: String,

	#[serde(rename = "callbackcontract")]
	pub callback_contract: H160,

	#[serde(rename = "callbackmethod")]
	pub callback_method: String,

	#[serde(rename = "userdata")]
	pub user_data: String,
}

impl OracleRequest {
	pub fn new(
		request_id: i32,
		original_transaction_hash: H256,
		gas_for_response: i32,
		url: String,
		filter: String,
		callback_contract: H160,
		callback_method: String,
		user_data: String,
	) -> Self {
		Self {
			request_id,
			original_transaction_hash,
			gas_for_response,
			url,
			filter,
			callback_contract,
			callback_method,
			user_data,
		}
	}
}

impl Hash for OracleRequest {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.request_id.hash(state);
		self.original_transaction_hash.hash(state);
		self.gas_for_response.hash(state);
		self.url.hash(state);
		self.filter.hash(state);
		self.callback_contract.hash(state);
		self.callback_method.hash(state);
		self.user_data.hash(state);
	}
}
