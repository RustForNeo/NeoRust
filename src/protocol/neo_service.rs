use crate::protocol::core::request::NeoRequest;

pub trait NeoService {
    async fn send<T, U>(&self, request: &NeoRequest<T, U>) -> Result<T, Err>;
    fn close(&self);
}