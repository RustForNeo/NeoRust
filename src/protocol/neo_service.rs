use crate::{
	neo_error::NeoError,
	protocol::core::{request::NeoRequest, response::NeoResponse},
};
use async_trait::async_trait;

#[async_trait]
pub trait NeoService: Send + Sync {
	fn send<'a, T>(&'a self, request: &'a NeoRequest<T>) -> Result<NeoResponse<T>, NeoError>;
	fn close(&self);
}
