use crate::wallet::MnemonicBuilderError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
	#[error("Account state error: {0}")]
	AccountState(String),
	#[error("No default account")]
	NoDefaultAccount,
	#[error("No key pair")]
	NoKeyPair,
	#[error("Invalid key pair")]
	SignHashError,
	#[error(transparent)]
	Bip32Error(#[from] coins_bip32::Bip32Error),
	#[error(transparent)]
	StdError(#[from] std::io::Error),
	#[error(transparent)]
	MnemonicError(#[from] coins_bip39::MnemonicError),
	#[error(transparent)]
	MnemonicBuilderError(#[from] MnemonicBuilderError),
	#[error(transparent)]
	CryptoError(#[from] neo_crypto::error::CryptoError),
}
