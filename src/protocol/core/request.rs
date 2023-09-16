#![feature(atomic_from_ptr, pointer_is_aligned)]
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::neo_error::NeoError;
use crate::protocol::core::response;
use crate::protocol::core::response::{ResponseTrait};
use crate::protocol::neo_rust::NeoRust;

#[derive(Serialize, Deserialize)]
pub struct NeoRequest<T, U> where T: ResponseTrait<U>{
    jsonrpc: &'static str,
    method: String,
    params: Vec<Value>,
    id: u64,
}

impl<T, U> NeoRequest<T, U>
    where
        T: ResponseTrait<U>,
        U: Serialize+Deserialize
{

    pub fn new(method: &str, params: Vec<Value>) -> Self {
        Self {
            jsonrpc: "2.0",
            method: method.to_string(),
            params,
            id: next_id(),
        }
    }

    pub(crate) fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub async fn send<T, U>(&self) -> Result<U, NeoError> {
        let response = NeoRust::instance().get_neo_service().send(self).await.unwrap();
        response.get_result()
    }
}

// Generate unique ID
fn next_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}