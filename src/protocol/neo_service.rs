use crate::protocol::core::request::Request;

pub trait NeoService {
    async fn send<T, U>(&self, request: Request<T, U>) -> Result<T, Err>;

    fn close(&self);
}