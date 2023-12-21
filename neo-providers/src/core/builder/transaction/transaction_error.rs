use std::error;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransactionError {
	#[error("Script format error: {0}")]
	ScriptFormat(String),
	#[error("Signer configuration error: {0}")]
	SignerConfiguration(String),
	#[error("Invalid nonce")]
	InvalidNonce,
	#[error("Invalid block")]
	InvalidBlock,
	#[error("Invalid transaction")]
	InvalidTransaction,
	#[error("Too many signers")]
	TooManySigners,
	#[error("Duplicate signer")]
	DuplicateSigner,
	#[error("No signers")]
	NoSigners,
	#[error("No script")]
	NoScript,
	#[error("Empty script")]
	EmptyScript,
	#[error("Invalid sender")]
	InvalidSender,
	#[error("Invalid state:{0}")]
	IllegalState(String),
	#[error("Transaction too large")]
	TxTooLarge,
	#[error("Transaction configuration error: {0}")]
	TransactionConfiguration(String),
}
