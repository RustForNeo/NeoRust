use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ContractError {
    InvalidNeoName(String),
    InvalidNeoNameServiceRoot(String),
    UnexpectedReturnType(String, Option<Vec<String>>),
    UnresolvableDomainName(String),
    RuntimeError(String),
    InvalidStateError(String),
    InvalidArgError(String),
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContractError::InvalidNeoName(name) => {
                write!(f, "'{}' is not a valid NNS name.", name)
            }
            ContractError::InvalidNeoNameServiceRoot(root) => {
                write!(f, "'{}' is not a valid NNS root.", root)
            }
            ContractError::UnexpectedReturnType(r#type, expected) => {
                if let Some(expected) = expected {
                    write!(f, "Got stack item of type {} but expected {:?}.", r#type, expected)
                } else {
                    write!(f, "{}", r#type)
                }
            }
            ContractError::UnresolvableDomainName(name) => {
                write!(f, "The provided domain name '{}' could not be resolved.", name)
            },
            ContractError::RuntimeError(msg) => {
                write!(f, "Runtime error: {}", msg)
            }
            ContractError::InvalidStateError(msg) => {
                write!(f, "Invalid state error: {}", msg)
            }
            ContractError::InvalidArgError(msg) => {
                write!(f, "Invalid argument error: {}", msg)
            }
        }
    }
}

impl Error for ContractError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ContractError::InvalidNeoName(_) => None,
            ContractError::InvalidNeoNameServiceRoot(_) => None,
            ContractError::UnexpectedReturnType(_, _) => None,
            ContractError::UnresolvableDomainName(_) => None,
            ContractError::RuntimeError(_) => None,
            ContractError::InvalidStateError(_) => None,
            ContractError::InvalidArgError(_) => None,
        }
    }
    fn description(&self) -> &str {
        match self {
            ContractError::InvalidNeoName(_) => "Invalid NNS name",
            ContractError::InvalidNeoNameServiceRoot(_) => "Invalid NNS root",
            ContractError::UnexpectedReturnType(_, _) => "Unexpected return type",
            ContractError::UnresolvableDomainName(_) => "Unresolvable domain name",
            ContractError::RuntimeError(_) => "Runtime error",
            ContractError::InvalidStateError(_) => "Invalid state error",
            ContractError::InvalidArgError(_) => "Invalid argument error",
        }
    }
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            ContractError::InvalidNeoName(_) => None,
            ContractError::InvalidNeoNameServiceRoot(_) => None,
            ContractError::UnexpectedReturnType(_, _) => None,
            ContractError::UnresolvableDomainName(_) => None,
            ContractError::RuntimeError(_) => None,
            ContractError::InvalidStateError(_) => None,
            ContractError::InvalidArgError(_) => None,
        }
    }
}