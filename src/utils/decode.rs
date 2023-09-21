use serde::{Deserialize, Serialize};
use serde_json::Value;

use derive_more::{AsRef, Deref, Display, Index, IndexMut, IntoIterator};
use getset::{CopyGetters, Getters};
use std::{convert::TryFrom, str::FromStr};

#[derive(
	Debug,
	Serialize,
	Deserialize,
	AsRef,
	Deref,
	IntoIterator,
	Index,
	IndexMut,
	Getters,
	CopyGetters,
	Default,
)]
#[getset(get = "pub", set = "pub")]
pub struct Bytes {
	bytes: Vec<u8>,
}

impl FromStr for Bytes {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		base64::decode(s).map(Self::from).map_err(|_| "Invalid base64 string")
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct SafeDecode<T: FromStr> {
	value: T,
}

impl<T: FromStr> SafeDecode<T> {
	fn new(s: &str) -> Result<Self, T::Err> {
		let value = T::from_str(s).unwrap();
		Ok(Self { value })
	}
}

impl From<Vec<u8>> for Bytes {
	fn from(bytes: Vec<u8>) -> Self {
		Self { bytes }
	}
}
impl<T: FromStr> From<String> for SafeDecode<T> {
	fn from(s: String) -> Self {
		Self::new(&s).unwrap()
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct StringDecode<T: FromStr> {
	#[serde(with = "SafeDecode")]
	value: T,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawResponse {
	data: String,
}

impl RawResponse {
	fn extract_string(&self) -> String {
		self.data.clone()
	}
}
