use reqwest::{Client, Result};
use std::collections::HashMap;
use futures::TryFutureExt;
use crate::protocol::neo_rust::NeoRust;
use crate::protocol::protocol_error::ProtocolError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HttpService {
    client: NeoRust,
    base_url: String,
    headers: HashMap<String, String>,
    include_raw: bool,
}

impl HttpService {

    pub fn new(base_url: String) -> Self {
        Self {
            client: NeoRust ::new(),
            base_url,
            headers: HashMap::new(),
            include_raw: false,
        }
    }
    pub fn include_raw_responses(&mut self, enable: bool) {
        self.include_raw = enable;
    }

    pub fn post<T: serde::de::DeserializeOwned>(&self, payload: &T) -> Result<String> {
        let url = format!("{}", self.base_url);

        let mut request = self.client.post(url);

        request = request.json(payload);

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        request.send().and_then(|res| res.text())
    }

    // pub fn post<T>(&self, payload: &T) -> Result<String, ServiceError> {
    //     self.client.post(url, payload)
    //         .and_then(|res| res.text())
    //         .map_err(|e| ServiceError::HttpError(e))
    // }

    pub async fn post_async<T>(&self, payload: &T) -> Result<String, ProtocolError> {
        // send post request
        let res = self.client.post(url, payload).await?;

        // get text responses
        res.text().await.map_err(ProtocolError::HttpError)
    }
    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

}