use crate::{
	neo_error::NeoError,
	protocol::core::{request::NeoRequest, response::NeoResponse},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait NeoService: Send + Sync {
	fn send<'a, T: Deserialize<'a> + Serialize>(
		&self,
		request: &NeoRequest<T>,
	) -> Result<NeoResponse<T>, NeoError>;
	fn close(&self);
}
