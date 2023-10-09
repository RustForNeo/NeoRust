use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Nep2Error {
	#[error("Invalid passphrase: {0}")]
	InvalidPassphrase(String),
	#[error("Invalid format: {0}")]
	InvalidFormat(String),
}
