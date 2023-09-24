#![feature(atomic_from_ptr, pointer_is_aligned)]

use crate::{
	neo_error::NeoError,
	protocol::{core::response::ResponseTrait, neo_service::NeoService},
	NEO_INSTANCE,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::{
	marker::PhantomData,
	sync::atomic::{AtomicU64, Ordering},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct NeoRequest<T> {
	jsonrpc: &'static str,
	method: String,
	params: Vec<Value>,
	id: u64,
	_marker: PhantomData<T>,
}

unsafe impl<'a, T> Send for NeoRequest<T> {}

unsafe impl<'a, T> Sync for NeoRequest<T> {}

impl<T> NeoRequest<T>
where
	T: Serialize + DeserializeOwned + Clone,
{
	pub fn new(method: &str, params: Vec<Value>) -> Self {
		Self {
			jsonrpc: "2.0",
			method: method.to_string(),
			params,
			id: next_id(),
			_marker: Default::default(),
		}
	}

	pub(crate) fn to_json(&self) -> String {
		serde_json::to_string(self).unwrap()
	}

	pub async fn request(&self) -> Result<T, NeoError> {
		let neo_rust_instance_guard = { NEO_INSTANCE.read().unwrap().get_neo_service().clone() };
		let response = neo_rust_instance_guard.send(&self).await.unwrap();

		response.get_result()
	}
}

// Generate unique ID
fn next_id() -> u64 {
	static COUNTER: AtomicU64 = AtomicU64::new(1);
	COUNTER.fetch_add(1, Ordering::Relaxed)
}
