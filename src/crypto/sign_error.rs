use thiserror::Error;

#[derive(Error, Debug)]
pub enum SignError {
	#[error("Header byte out of range: {0}")]
	HeaderOutOfRange(u8),
	#[error("Could not recover public key from signature")]
	RecoverFailed,
}
