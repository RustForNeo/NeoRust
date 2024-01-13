use crate::wallet::MnemonicBuilderError;
use coins_bip39::MnemonicError;
use eth_keystore::KeystoreError;
use p256::ecdsa;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
	#[error("Account state error: {0}")]
	AccountState(String),
	#[error("No key pair")]
	NoKeyPair,
	/// Underlying eth keystore error
	#[cfg(not(target_arch = "wasm32"))]
	#[error(transparent)]
	NeoKeystoreError(#[from] KeystoreError),
	/// Error propagated from p256's ECDSA module
	#[error(transparent)]
	EcdsaError(#[from] ecdsa::Error),
	/// Error propagated from the hex crate.
	#[error(transparent)]
	HexError(#[from] hex::FromHexError),
	/// Error propagated by IO operations
	#[error(transparent)]
	IoError(#[from] std::io::Error),
	#[error("No default account")]
	NoDefaultAccount,
	#[error("Invalid key pair")]
	SignHashError,
	#[error(transparent)]
	Bip32Error(#[from] coins_bip32::Bip32Error),
	#[error(transparent)]
	MnemonicError(#[from] MnemonicError),
	#[error(transparent)]
	MnemonicBuilderError(#[from] MnemonicBuilderError),
	#[error(transparent)]
	CryptoError(#[from] neo_crypto::error::CryptoError),
}
