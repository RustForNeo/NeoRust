use crate::protocol::{
	core::{
		request::NeoRequest,
		response::{NeoResponse, ResponseTrait},
	},
	neo_service::NeoService,
};
use reqwest::{Client, Response, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub struct HttpService {
	url: Url,
	client: Client,
	headers: HashMap<String, String>,
	include_raw_responses: bool,
}

impl HttpService {
	pub const JSON_MEDIA_TYPE: &'static str = "application/json; charset=utf-8";
	pub const DEFAULT_URL: &'static str = "http://localhost:10333/";

	pub fn new(url: Url, include_raw_responses: bool) -> Self {
		HttpService { url, client: Client::new(), headers: HashMap::new(), include_raw_responses }
	}

	pub fn add_header(&mut self, key: String, value: String) {
		self.headers.insert(key, value);
	}

	pub fn add_headers(&mut self, headers: HashMap<String, String>) {
		self.headers.extend(headers);
	}

	pub fn set_client(&mut self, client: Client) {
		self.client = client;
	}
}

impl NeoService for HttpService {
	async fn send<T, U>(&self, request: &NeoRequest<T, U>) -> Result<T, Err>
	where
		T: ResponseTrait<U>,
		U: Serialize + Deserialize,
	{
		let mut client = self.client.post(self.url.clone());

		client = client.header("Content-Type", Self::JSON_MEDIA_TYPE).json(&request);

		for (key, value) in &self.headers {
			client = client.header(key, value);
		}
		client = client.body(&request.to_json());

		let response = client.send().await?;

		if response.status().is_success() {
			if self.include_raw_responses {
				// Return raw response along with bytes
				// let (bytes, response) = http_service.perform_io(payload).await?;
				// let result = response.json::<NeoResponse<U>>().await?;
			}

			let result = response.json::<NeoResponse<U>>().await?;
			Ok(result.get_result())
		} else {
			let result = response.json::<Value>().await?;
			Err(result)
		}
		.expect("Failed to parse response")
	}

	fn close(&self) {
		unimplemented!()
	}
}
