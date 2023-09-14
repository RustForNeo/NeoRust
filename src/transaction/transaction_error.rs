use std::error;

#[derive(Debug)]
pub enum TransactionError {
    ScriptFormat(String),
    SignerConfiguration(String),
    InvalidNonce,
    InvalidBlock,
    InvalidTransaction,
    TooManySigners,
    DuplicateSigner,
    NoSigners,
    NoScript,
    EmptyScript,
    InvalidSender,
    TxTooLarge,
    TransactionConfiguration(String),
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TransactionError::ScriptFormat(msg) =>
                write!(f, "Script format error: {}", msg),

            TransactionError::SignerConfiguration(msg) =>
                write!(f, "Signer configuration error: {}", msg),
            TransactionError::InvalidNonce =>
                write!(f, "Invalid nonce"),
            TransactionError::InvalidBlock =>
                write!(f, "Invalid block"),
            TransactionError::InvalidTransaction =>
                write!(f, "Invalid transaction"),
            TransactionError::TooManySigners =>
                write!(f, "Too many signers"),
            TransactionError::DuplicateSigner =>
                write!(f, "Duplicate signer"),
            TransactionError::NoSigners =>
                write!(f, "No signers"),
            TransactionError::NoScript =>
                write!(f, "No script"),
            TransactionError::EmptyScript =>
                write!(f, "Empty script"),
            TransactionError::InvalidSender =>
                write!(f, "Invalid sender"),
            TransactionError::TxTooLarge =>
                write!(f, "Transaction too large"),
            TransactionError::TransactionConfiguration(msg) =>
                write!(f, "Transaction configuration error: {}", msg),
        }
    }
}

impl std::error::Error for TransactionError {

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // TransactionErrors do not wrap other errors
        None
    }

    fn description(&self) -> &str {
        // Use Display implementation to generate description
        match self {
            TransactionError::ScriptFormat(msg) => msg,
            TransactionError::SignerConfiguration(msg) => msg,
            TransactionError::InvalidNonce => "Invalid nonce",
            TransactionError::InvalidBlock => "Invalid block",
            TransactionError::InvalidTransaction => "Invalid transaction",
            TransactionError::TooManySigners => "Too many signers",
            TransactionError::DuplicateSigner => "Duplicate signer",
            TransactionError::NoSigners => "No signers",
            TransactionError::NoScript => "No script",
            TransactionError::EmptyScript => "Empty script",
            TransactionError::InvalidSender => "Invalid sender",
            TransactionError::TxTooLarge => "Transaction too large",
            TransactionError::TransactionConfiguration(msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        // Underlying cause is ourselves
        Some(self)
    }
}