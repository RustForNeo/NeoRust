use serde::{Serialize, Deserialize};
use crate::neo_error::NeoError;


pub trait ResponseTrait<T> where T:Serialize+Deserialize{
    fn get_result(self) -> Result<T, NeoError>;
}

#[derive(Serialize, Deserialize)]
pub struct NeoResponse<T> {
    jsonrpc: &'static str,
    id: u64,
    result: Option<T>,
    error: Option<Error>,
}

#[derive(Serialize, Deserialize)]
pub struct Error {
    code: i32,
    message: String,
    data: Option<String>,
}

impl<T> NeoResponse<T>
    where
        T: Serialize+Deserialize,
{

    fn new(result: T) -> Self {
        Self {
            jsonrpc: "2.0",
            id: 0,
            result: Some(result),
            error: None
        }
    }

    fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

impl<T> ResponseTrait<T> for NeoResponse<T>
where T: Serialize+Deserialize{
    fn get_result(self) -> Result<T, NeoError> {
        match self.error {
            Some(err) => Err(NeoError::InvalidData(err.message)),
            None => Ok(self.result.unwrap()),
        }
    }
}