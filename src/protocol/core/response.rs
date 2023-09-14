use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;

#[derive(Serialize, Deserialize)]
pub struct Response<T> {

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

impl<T> Response<T>
    where
        T: DeserializeOwned,
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

    fn get_result(self) -> Result<T, Error> {
        match self.error {
            Some(err) => Err(err),
            None => Ok(self.result.unwrap()),
        }
    }

}