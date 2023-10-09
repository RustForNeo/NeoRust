use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuilderError {
	InvalidScript,
	InvalidOperation,
	InvalidArgument,
	InvalidState,
	InvalidInvocation,
	StackOverflow,
	OutOfGas,
	OutOfMemory,
	OutOfCycles,
	UnknownError,
}
