use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Hash, Clone)]
pub enum WalletError {
	#[error("Account state error: {0}")]
	AccountState(String),
	#[error("No default account")]
	NoDefaultAccount,
	#[error("No key pair")]
	NoKeyPair,
}