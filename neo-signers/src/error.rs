use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignerError {
	#[error("Invalid passphrase: {0}")]
	InvalidPassphrase(String),
	#[error("Invalid address")]
	InvalidAddress,
	#[error(transparent)]
	BuilderError(#[from] crate::builder::error::BuilderError),
	#[error(transparent)]
	WalletError(#[from] crate::wallet::wallet_error::WalletError),
}
