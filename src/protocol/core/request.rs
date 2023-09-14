#![feature(atomic_from_ptr, pointer_is_aligned)]
use std::sync::atomic::{AtomicU64, Ordering};
use std::error::Error;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct Request<T> {
    jsonrpc: &'static str,
    method: String,
    params: Vec<Value>,
    id: u64,
}

impl<T> Request<T>
    where
        T: DeserializeOwned,
{

    fn new(method: &str, params: Vec<Value>) -> Self {
        Self {
            jsonrpc: "2.0",
            method: method.to_string(),
            params,
            id: next_id(),
        }
    }

    async fn send(&self, client: &Client) -> Result<T, dyn Error> {
        let response = client.post(self).await?;
        let result = response.json::<T>().await?;
        Ok(result)
    }

}

// Generate unique ID
fn next_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}