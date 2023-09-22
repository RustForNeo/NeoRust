#![feature(atomic_from_ptr, pointer_is_aligned)]

use crate::{
	neo_error::NeoError,
	protocol::{
		core::response::ResponseTrait, http_service::HttpService, neo_rust::NeoRust,
		neo_service::NeoService,
	},
};
use futures::future::ready;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
	marker::PhantomData,
	sync::atomic::{AtomicU64, Ordering},
};

#[derive(Serialize, Deserialize)]
pub struct NeoRequest<'a, T> {
	jsonrpc: &'static str,
	method: String,
	params: Vec<Value>,
	id: u64,
	_marker: PhantomData<&'a T>,
}

impl<'a, T> NeoRequest<'a, T>
where
	T: Serialize + Deserialize<'a>,
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
		let neo_rust_instance_guard = NeoRust::instance();
		let service = neo_rust_instance_guard.get_neo_service();
		let cloned_self = self.clone();
		let response = service.send(&cloned_self).unwrap(); // No async call here since send isn't marked as async

		response.get_result()
	}
}

// Generate unique ID
fn next_id() -> u64 {
	static COUNTER: AtomicU64 = AtomicU64::new(1);
	COUNTER.fetch_add(1, Ordering::Relaxed)
}
