#[derive(Debug)]
pub enum NeoError {
    IllegalArgument(String),
    Deserialization(String),
    IllegalState(String),
    IndexOutOfBounds(String),
    InvalidConfiguration(String),
    Runtime(String),
    InvalidData(String),
    UnsupportedOperation(String),
    Transaction(String),
    InvalidScript(String),
    InvalidFormat,
}

impl std::fmt::Display for NeoError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NeoError::IllegalArgument(msg) => {
                write!(f, "Illegal argument: {}", msg)
            },
            NeoError::Deserialization(msg) => {
                write!(f, "Deserialization error: {}", msg)
            },
            NeoError::IllegalState(msg) => {
                write!(f, "Illegal state: {}", msg)
            },
            NeoError::IndexOutOfBounds(msg) => {
                write!(f, "Index out of bounds: {}", msg)
            },
            NeoError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            },
            NeoError::Runtime(msg) => {
                write!(f, "Runtime error: {}", msg)
            },
            NeoError::InvalidData(msg) => {
                write!(f, "Invalid data: {}", msg)
            },
            NeoError::UnsupportedOperation(msg) => {
                write!(f, "Unsupported operation: {}", msg)
            },
            NeoError::Transaction(msg) => {
                write!(f, "Transaction error: {}", msg)
            },
            NeoError::InvalidScript(msg) => {
                write!(f, "Invalid script: {}", msg)
            },
            NeoError::InvalidFormat => {
                write!(f, "Invalid format")
            },

        }
    }
}

impl std::error::Error for NeoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // NeoErrors do not wrap other errors
        None
    }

    fn description(&self) -> &str {
        // Use the Display implementation to generate description
        match self {
            NeoError::IllegalArgument(msg) => msg,
            NeoError::Deserialization(msg) => msg,
            NeoError::IllegalState(msg) => msg,
            NeoError::IndexOutOfBounds(msg) => msg,
            NeoError::InvalidConfiguration(msg) => msg,
            NeoError::Runtime(msg) => msg,
            NeoError::InvalidData(msg) => msg,
            NeoError::UnsupportedOperation(msg) => msg,
            NeoError::Transaction(msg) => msg,
            NeoError::InvalidScript(msg) => msg,
            NeoError::InvalidFormat => "Invalid format",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        // The underlying cause is ourselves
        Some(self)
    }
}