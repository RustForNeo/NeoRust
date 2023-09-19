use std::error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionError {
	#[error("Script format error: {0}")]
	ScriptFormat(String),
	#[error("Signer configuration error: {0}")]
	SignerConfiguration(String),
	#[error("Invalid nonce")]
	InvalidNonce,
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
	#[error("Transaction too large")]
	TxTooLarge,
	#[error("Transaction configuration error: {0}")]
	TransactionConfiguration(String),
}
