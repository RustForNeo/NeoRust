use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::convert::TryFrom;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
struct Bytes(Vec<u8>);

impl FromStr for Bytes {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		base64::decode(s).map(Bytes).map_err(|_| "Invalid base64 string")
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct SafeDecode<T: FromStr> {
	value: T,
}

impl<T: FromStr> SafeDecode<T> {
	fn new(s: &str) -> Result<Self, T::Err> {
		let value = T::from_str(s)?;
		Ok(Self { value })
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

// Custom deserializer for RawResponse to extract string
impl<'de> Deserialize<'de> for RawResponse {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let v = Value::deserialize(deserializer)?;
		let data = v.as_str().unwrap().to_owned();
		Ok(RawResponse { data })
	}
}
