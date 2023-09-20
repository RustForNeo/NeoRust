#![feature(atomic_from_ptr, pointer_is_aligned)]
use crate::{
	neo_error::NeoError,
	protocol::{core::response::ResponseTrait, neo_rust::NeoRust},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Serialize, Deserialize)]
pub struct NeoRequest<'a, T> {
	jsonrpc: &'static str,
	method: String,
	params: Vec<Value>,
	id: u64,
}

impl<'a, T> NeoRequest<'a, T>
where
	T: Serialize + Deserialize<'a>,
{
	pub fn new(method: &str, params: Vec<Value>) -> Self {
		Self { jsonrpc: "2.0", method: method.to_string(), params, id: next_id() }
	}

	pub(crate) fn to_json(&self) -> String {
		serde_json::to_string(self).unwrap()
	}

	pub async fn request(&self) -> Result<T, NeoError> {
		let response = NeoRust::instance().get_neo_service().send(self).await.unwrap();
		response.get_result()
	}
}

// Generate unique ID
fn next_id() -> u64 {
	static COUNTER: AtomicU64 = AtomicU64::new(1);
	COUNTER.fetch_add(1, Ordering::Relaxed)
}
