use crate::{neo_error::NeoError, protocol::core::request::NeoRequest};
use async_trait::async_trait;

#[async_trait]
pub trait NeoService {
	fn send<T, U>(&self, request: &NeoRequest<T, U>) -> Result<T, NeoError>;
	fn close(&self);
}
