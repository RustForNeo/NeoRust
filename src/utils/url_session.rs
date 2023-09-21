// URLSession.rs

use reqwest::blocking::{Client, Request};

pub struct URLSession;

impl URLSession {
	pub async fn data(
		&self,
		request: Request,
	) -> Result<(Vec<u8>, reqwest::Response), reqwest::Error> {
		let client = Client::new();
		let response = client.execute(request).await.unwrap();
		let data = response.bytes().await.unwrap();

		Ok((data, response))
	}
}
