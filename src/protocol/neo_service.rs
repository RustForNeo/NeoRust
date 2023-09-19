use crate::protocol::core::request::NeoRequest;

pub trait NeoService {
	fn send<T, U>(&self, request: &NeoRequest<T, U>) -> Result<T, Err>;
	fn close(&self);
}
