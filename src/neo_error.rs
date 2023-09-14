#[derive(Debug)]
pub enum NeoRustError {
    IllegalArgument(String),
    Deserialization(String),
    IllegalState(String),
    IndexOutOfBounds(String),
    InvalidConfiguration(String),
    Runtime(String),
    InvalidData(String),
    UnsupportedOperation(String),
    Transaction(String),
}

impl std::fmt::Display for NeoRustError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NeoRustError::IllegalArgument(msg) => {
                write!(f, "Illegal argument: {}", msg)
            },
            NeoRustError::Deserialization(msg) => {
                write!(f, "Deserialization error: {}", msg)
            },
            NeoRustError::IllegalState(msg) => {
                write!(f, "Illegal state: {}", msg)
            },
            NeoRustError::IndexOutOfBounds(msg) => {
                write!(f, "Index out of bounds: {}", msg)
            },
            NeoRustError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            },
            NeoRustError::Runtime(msg) => {
                write!(f, "Runtime error: {}", msg)
            },
            NeoRustError::InvalidData(msg) => {
                write!(f, "Invalid data: {}", msg)
            },
            NeoRustError::UnsupportedOperation(msg) => {
                write!(f, "Unsupported operation: {}", msg)
            },
            NeoRustError::Transaction(msg) => {
                write!(f, "Transaction error: {}", msg)
            },
        }
    }
}

impl std::error::Error for NeoRustError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // NeoRustErrors do not wrap other errors
        None
    }

    fn description(&self) -> &str {
        // Use the Display implementation to generate description
        match self {
            NeoRustError::IllegalArgument(msg) => msg,
            NeoRustError::Deserialization(msg) => msg,
            NeoRustError::IllegalState(msg) => msg,
            NeoRustError::IndexOutOfBounds(msg) => msg,
            NeoRustError::InvalidConfiguration(msg) => msg,
            NeoRustError::Runtime(msg) => msg,
            NeoRustError::InvalidData(msg) => msg,
            NeoRustError::UnsupportedOperation(msg) => msg,
            NeoRustError::Transaction(msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        // The underlying cause is ourselves
        Some(self)
    }
}