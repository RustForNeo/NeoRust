use crate::{
	neo_error::NeoError,
	protocol::core::{request::NeoRequest, response::NeoResponse},
};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
pub trait NeoService: Send + Sync {
	async fn send<T: DeserializeOwned + Serialize + Clone>(
		&self,
		request: &NeoRequest<T>,
	) -> Result<NeoResponse<T>, NeoError>;
	fn close(&self);
}
