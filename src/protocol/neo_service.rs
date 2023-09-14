use crate::protocol::core::request::Request;

pub trait NeoService {
    async fn send<T: Response<U>, U>(&self, request: Request<T, U>) -> Result<T, Error>;

    fn close(&self);
}