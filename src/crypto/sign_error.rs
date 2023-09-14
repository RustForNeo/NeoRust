#[derive(Debug)]
pub enum SignError {
    HeaderOutOfRange(u8),
    RecoverFailed,
}

impl std::fmt::Display for SignError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SignError::HeaderOutOfRange(byte) => {
                write!(f, "Header byte out of range: {}", hex::encode(byte))
            },
            SignError::RecoverFailed => {
                write!(f, "Could not recover public key from signature")
            }
        }
    }
}

impl std::error::Error for SignError {
fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // SignErrors do not wrap other errors
        None
    }

    fn description(&self) -> &str {
        // Use the Display implementation to generate description
        match self {
            SignError::HeaderOutOfRange(byte) => "Header byte out of range",
            SignError::RecoverFailed => "Could not recover public key from signature",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        // The underlying cause is ourselves
        Some(self)
    }
}