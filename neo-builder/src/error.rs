use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuilderError {
	#[error("Invalid operation")]
	InvalidScript(String),
	#[error("Invalid operation")]
	InvalidOperation,
	#[error("Invalid argument")]
	InvalidArgument,
	#[error("Invalid state")]
	InvalidState,
	#[error("Invalid invocation")]
	InvalidInvocation,
	#[error("Stack overflow")]
	StackOverflow,
	#[error("Out of gas")]
	OutOfGas,
	#[error("Out of memory")]
	OutOfMemory,
	#[error("Out of cycles")]
	OutOfCycles,
	#[error("UnknownError")]
	UnknownError,
	#[error("Unsupported operation: {0}")]
	UnsupportedOperation(String),
	#[error("Invalid signer configuration: {0}")]
	SignerConfiguration(String),
	#[error("Invalid transaction configuration: {0}")]
	TransactionConfiguration(String),
	#[error("Invalid configuration: {0}")]
	InvalidConfiguration(String),
	#[error("Too many signers: {0}")]
	TooManySigners(String),
	#[error("Illegal state: {0}")]
	IllegalState(String),
	#[error("Illegal argument: {0}")]
	IllegalArgument(String),
}
