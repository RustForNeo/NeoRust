use hex::FromHexError;
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
	#[error(transparent)]
	FromHexError(#[from] FromHexError),
	#[error(transparent)]
	CryptoError(#[from] neo_crypto::error::CryptoError),
	#[error(transparent)]
	RustcFromHexError(#[from] rustc_serialize::hex::FromHexError),
	#[error(transparent)]
	TypeError(#[from] neo_types::error::TypeError),
}
