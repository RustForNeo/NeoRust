#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Nep2Error {
    InvalidPassphrase(String),
    InvalidFormat(String),
}

impl std::fmt::Display for Nep2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Nep2Error::InvalidPassphrase(msg) => {
                write!(f, "Invalid passphrase: {}", msg)
            },
            Nep2Error::InvalidFormat(msg) => {
                write!(f, "Invalid format: {}", msg)
            },
        }
    }
}

impl std::error::Error for Nep2Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Nep2Errors do not wrap other errors
        None
    }

    fn description(&self) -> &str {
        // Use the Display implementation to generate description
        match self {
            Nep2Error::InvalidPassphrase(msg) => msg,
            Nep2Error::InvalidFormat(msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        // The underlying cause is ourselves
        Some(self)
    }
}