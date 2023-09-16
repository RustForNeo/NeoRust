
use reqwest::{Client, Response, Url};
use std::collections::HashMap;
use serde_json::Value;
use crate::protocol::core::request::Request;
use crate::protocol::neo_service::NeoService;

pub struct HttpService {
    url: Url,
    client: reqwest::Client,
    headers: HashMap<String, String>,
    include_raw_responses: bool,
}

impl HttpService {
    pub const JSON_MEDIA_TYPE: &'static str = "application/json; charset=utf-8";
    pub const DEFAULT_URL: &'static str = "http://localhost:10333/";

    pub fn new(url: Url, include_raw_responses: bool) -> Self {
        HttpService {
            url,
            client: Client::new(),
            headers: HashMap::new(),
            include_raw_responses,
        }
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

impl NeoService for HttpService{
    async fn send<T, U>(&self,  request: Request<T, U>) -> std::result::Result<T, Err> {

        let mut request = self.client.post(self.url.clone());

        request = request
            .header("Content-Type", Self::JSON_MEDIA_TYPE)
            .json(&request);

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let result = response.json::<Response>().await?;
            Ok(result)
        } else {
            let result = response.json::<Value>().await?;
            Err(result)
        }

        if self.include_raw_responses {
            // Return raw response along with bytes
            // let (bytes, response) = http_service.perform_io(payload).await?;

        }

        Ok(bytes.to_vec())


    }

    fn close(&self) {
        unimplemented!()
    }
}