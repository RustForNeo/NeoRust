use crate::{
	neo_error::NeoError,
	protocol::core::{request::NeoRequest, response::NeoResponse},
};
use async_trait::async_trait;

#[async_trait]
pub trait NeoService {
	fn send<T>(&self, request: &NeoRequest<T>) -> Result<NeoResponse<T>, NeoError>;
	fn close(&self);
}
