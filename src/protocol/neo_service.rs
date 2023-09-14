use crate::protocol::core::request::Request;
use crate::protocol::core::response::Response;

pub trait NeoService {
    async fn send<T, U>(&self, request: Request<T, U>) -> Result<T, Err>;

    fn close(&self);
}