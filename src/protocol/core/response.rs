use crate::neo_error::NeoError;
use serde::{Deserialize, Serialize};
use std::{
	future::Future,
	pin::Pin,
	task::{Context, Poll},
};

pub trait ResponseTrait<'a, T>
where
	T: Serialize + Deserialize<'a>,
{
	fn get_result(self) -> Result<T, NeoError>;
}

#[derive(Serialize, Deserialize)]
pub struct NeoResponse<T> {
	jsonrpc: &'static str,
	id: u64,
	#[serde(skip_serializing_if = "Option::is_none")]
	result: Option<T>,
	#[serde(skip_serializing_if = "Option::is_none")]
	error: Option<Error>,
}

#[derive(Serialize, Deserialize)]
pub struct Error {
	code: i32,
	message: String,
	data: Option<String>,
}

impl<'a, T> NeoResponse<T>
where
	T: Serialize + Deserialize<'a>,
{
	fn new(result: T) -> Self {
		Self { jsonrpc: "2.0", id: 0, result: Some(result), error: None }
	}

	fn is_error(&self) -> bool {
		self.error.is_some()
	}
}

impl<'a, T> ResponseTrait<'a, T> for NeoResponse<T>
where
	T: Serialize + Deserialize<'a>,
{
	fn get_result(self) -> Result<T, NeoError> {
		match self.error {
			Some(err) => Err(NeoError::InvalidData(err.message)),
			None => Ok(self.result.unwrap()),
		}
	}
}

impl<T> Future for NeoResponse<T> {
	type Output = T;

	fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
		match &self.get_mut().result {
			Some(v) => Poll::Ready(v.clone()),
			None => Poll::Pending,
		}
	}
}
