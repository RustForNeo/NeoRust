//! Specific helper functions for loading an offline P256 Private Key stored on disk
use crate::wallet::{mnemonic::MnemonicBuilderError, wallet::Wallet};
use coins_bip32::Bip32Error;
use coins_bip39::MnemonicError;
use derive_more::Display;
#[cfg(not(target_arch = "wasm32"))]
use elliptic_curve::rand_core;
use eth_keystore::KeystoreError;
use neo_crypto::{error::CryptoError, keys::Secp256r1PrivateKey};
use neo_types::{address_or_scripthash::AddressOrScriptHash, secret_key_to_script_hash};
use p256::{ecdsa, ecdsa::signature::rand_core::CryptoRng};
use rand::{rngs::OsRng, Rng};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug, Display)]
/// Error thrown by the Wallet module
pub enum WalletError {
	/// Error propagated from the BIP-32 crate
	#[error(transparent)]
	Bip32Error(#[from] Bip32Error),
	/// Error propagated from the BIP-39 crate
	#[error(transparent)]
	Bip39Error(#[from] MnemonicError),
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
	/// Error propagated from the mnemonic builder module.
	#[error(transparent)]
	MnemonicBuilderError(#[from] MnemonicBuilderError),
	// #[error(transparent)]
	NoDefaultAccount,
	#[error(transparent)]
	CryptoError(#[from] CryptoError),
}

impl Wallet {
	/// Creates a new random encrypted JSON with the provided password and stores it in the
	/// provided directory. Returns a tuple (Wallet, String) of the wallet instance for the
	/// keystore with its random UUID. Accepts an optional name for the keystore file. If `None`,
	/// the keystore is stored as the stringified UUID.
	#[cfg(not(target_arch = "wasm32"))]
	pub fn new_keystore<P, R, S>(
		dir: P,
		rng: &mut R,
		password: S,
		name: Option<&str>,
	) -> Result<(Self, String), WalletError>
	where
		P: AsRef<Path>,
		R: Rng + CryptoRng + rand_core::CryptoRng,
		S: AsRef<[u8]>,
	{
		let (secret, uuid) = eth_keystore::new(dir, rng, password, name)?;
		let signer = Secp256r1PrivateKey::from_bytes(secret.as_slice().into())?;
		let address = secret_key_to_script_hash(&signer);
		Ok((
			Self { signer, address: AddressOrScriptHash::ScriptHash(address), network_magic: 1 },
			uuid,
		))
	}

	/// Decrypts an encrypted JSON from the provided path to construct a Wallet instance
	#[cfg(not(target_arch = "wasm32"))]
	pub fn decrypt_keystore<P, S>(keypath: P, password: S) -> Result<Self, WalletError>
	where
		P: AsRef<Path>,
		S: AsRef<[u8]>,
	{
		let secret = eth_keystore::decrypt_key(keypath, password)?;
		let signer = Secp256r1PrivateKey::from_bytes(secret.as_slice().into())?;
		let address = secret_key_to_script_hash(&signer);
		Ok(Self { signer, address: AddressOrScriptHash::ScriptHash(address), network_magic: 1 })
	}

	/// Creates a new encrypted JSON with the provided private key and password and stores it in the
	/// provided directory. Returns a tuple (Wallet, String) of the wallet instance for the
	/// keystore with its random UUID. Accepts an optional name for the keystore file. If `None`,
	/// the keystore is stored as the stringified UUID.
	#[cfg(not(target_arch = "wasm32"))]
	pub fn encrypt_keystore<P, R, B, S>(
		keypath: P,
		rng: &mut R,
		pk: B,
		password: S,
		name: Option<&str>,
	) -> Result<(Self, String), WalletError>
	where
		P: AsRef<Path>,
		R: Rng + CryptoRng,
		B: AsRef<[u8]>,
		S: AsRef<[u8]>,
	{
		let uuid = eth_keystore::encrypt_key(keypath, rng, &pk, password, name)?;
		let signer = Secp256r1PrivateKey::from_bytes(pk.as_ref())?;
		let address = secret_key_to_script_hash(&signer);
		Ok((
			Self { signer, address: AddressOrScriptHash::ScriptHash(address), network_magic: 1 },
			uuid,
		))
	}

	/// Creates a new random keypair seeded with the provided RNG
	pub fn new<R: Rng + CryptoRng>(rng: &mut R) -> Self {
		let mut rng = OsRng;
		let signer = Secp256r1PrivateKey::random(&mut rng);
		let address = secret_key_to_script_hash(&signer);
		Self { signer, address: AddressOrScriptHash::ScriptHash(address), network_magic: 1 }
	}

	/// Creates a new Wallet instance from a raw scalar value (big endian).
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, WalletError> {
		let signer = Secp256r1PrivateKey::from_bytes(bytes.into())?;
		let address = secret_key_to_script_hash(&signer);
		Ok(Self { signer, address: AddressOrScriptHash::ScriptHash(address), network_magic: 1 })
	}
}

impl PartialEq for Wallet {
	fn eq(&self, other: &Self) -> bool {
		self.signer.to_raw_bytes().to_vec().eq(&other.signer.to_raw_bytes().to_vec())
			&& self.address == other.address
			&& self.network_magic == other.network_magic
	}
}

impl From<Secp256r1PrivateKey> for Wallet {
	fn from(signer: Secp256r1PrivateKey) -> Self {
		let address = secret_key_to_script_hash(&signer);

		Self { signer, address: AddressOrScriptHash::ScriptHash(address), network_magic: 1 }
	}
}

impl FromStr for Wallet {
	type Err = WalletError;

	fn from_str(src: &str) -> Result<Self, Self::Err> {
		let src = hex::decode(src.strip_prefix("0X").unwrap_or(src))?;

		if src.len() != 32 {
			return Err(WalletError::HexError(hex::FromHexError::InvalidStringLength))
		}

		let sk = Secp256r1PrivateKey::from_bytes(src.as_slice().into())?;
		Ok(sk.into())
	}
}

impl TryFrom<&str> for Wallet {
	type Error = WalletError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		value.parse()
	}
}

impl TryFrom<String> for Wallet {
	type Error = WalletError;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		value.parse()
	}
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
	use super::*;
	use crate::{LocalWallet, Signer};
	use neo_types::hash_message;
	use tempfile::tempdir;

	#[test]
	fn parse_pk() {
		let s = "6f142508b4eea641e33cb2a0161221105086a84584c74245ca463a49effea30b";
		let _pk: Wallet = s.parse().unwrap();
	}

	#[test]
	fn parse_short_key() {
		let s = "6f142508b4eea641e33cb2a0161221105086a84584c74245ca463a49effea3";
		assert!(s.len() < 64);
		let pk = s.parse::<LocalWallet>().unwrap_err();
		match pk {
			WalletError::HexError(hex::FromHexError::InvalidStringLength) => {},
			_ => panic!("Unexpected error"),
		}
	}

	async fn test_encrypted_json_keystore(key: Wallet, uuid: &str, dir: &Path) {
		// sign a message using the given key
		let message = "Some data";
		let signature = key.sign_message(message).await.unwrap();

		// read from the encrypted JSON keystore and decrypt it, while validating that the
		// signatures produced by both the keys should match
		let path = Path::new(dir).join(uuid);
		let key2 = Wallet::decrypt_keystore(path.clone(), "randpsswd").unwrap();

		let signature2 = key2.sign_message(message).await.unwrap();
		assert_eq!(signature, signature2);

		std::fs::remove_file(&path).unwrap();
	}

	#[tokio::test]
	async fn encrypted_json_keystore_new() {
		// create and store an encrypted JSON keystore in this directory
		let dir = tempdir().unwrap();
		let mut rng = rand::thread_rng();
		let (key, uuid) = Wallet::new_keystore(&dir, &mut rng, "randpsswd", None).unwrap();

		test_encrypted_json_keystore(key, &uuid, dir.path()).await;
	}

	#[tokio::test]
	async fn encrypted_json_keystore_from_pk() {
		// create and store an encrypted JSON keystore in this directory
		let dir = tempdir().unwrap();
		let mut rng = rand::thread_rng();

		let private_key =
			hex::decode("6f142508b4eea641e33cb2a0161221105086a84584c74245ca463a49effea30b")
				.unwrap();

		let (key, uuid) =
			Wallet::encrypt_keystore(&dir, &mut rng, private_key, "randpsswd", None).unwrap();

		test_encrypted_json_keystore(key, &uuid, dir.path()).await;
	}

	#[tokio::test]
	async fn signs_msg() {
		let message = "Some data";
		let hash = hash_message(message.as_bytes());
		let key = Wallet::new(&mut rand::thread_rng());
		let address = key.address;

		// sign a message
		let signature = key.sign_message(message).await.unwrap();

		// ecrecover via the message will hash internally
		let recovered = signature.recover(message).unwrap();

		// if provided with a hash, it will skip hashing
		let recovered2 = signature.recover(hash).unwrap();

		// verifies the signature is produced by `address`
		signature.verify(message, address).unwrap();

		assert_eq!(recovered, address);
		assert_eq!(recovered2, address);
	}

	#[tokio::test]
	async fn signs_tx() {
		use crate::Transaction;
		// retrieved test vector from:
		// https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
		let tx: Transaction = TransactionRequest {
			from: None,
			to: Some("F0109fC8DF283027b6285cc889F5aA624EaC1F55".parse::<Address>().unwrap().into()),
			value: Some(1_000_000_000.into()),
			gas: Some(2_000_000.into()),
			nonce: Some(0.into()),
			gas_price: Some(21_000_000_000u128.into()),
			data: None,
			network_magic: Some(U64::one()),
		}
		.into();
		let wallet: Wallet = "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
			.parse()
			.unwrap();
		let wallet = wallet.with_network_magic(tx.network_magic().unwrap().as_u64());

		let sig = wallet.sign_transaction(&tx).await.unwrap();
		let sighash = tx.sighash();
		sig.verify(sighash, wallet.address).unwrap();
	}

	#[tokio::test]
	async fn signs_tx_empty_network_magic() {
		use crate::Transaction;
		use neo_types::TransactionRequest;
		// retrieved test vector from:
		// https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
		let tx: Transaction = TransactionRequest {
			from: None,
			to: Some("F0109fC8DF283027b6285cc889F5aA624EaC1F55".parse::<Address>().unwrap().into()),
			value: Some(1_000_000_000.into()),
			gas: Some(2_000_000.into()),
			nonce: Some(0.into()),
			gas_price: Some(21_000_000_000u128.into()),
			data: None,
			network_magic: None,
		}
		.into();
		let wallet: Wallet = "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
			.parse()
			.unwrap();
		let wallet = wallet.with_network_magic(1u64);

		// this should populate the tx network_magic as the signer's network_magic (1) before signing
		let sig = wallet.sign_transaction(&tx).await.unwrap();

		// since we initialize with None we need to re-set the network_magic for the sighash to be
		// correct
		let mut tx = tx;
		tx.set_network_magic(1);
		let sighash = tx.sighash();
		sig.verify(sighash, wallet.address).unwrap();
	}

	#[test]
	fn signs_tx_empty_network_magic_sync() {
		use crate::Transaction;
		use neo_types::TransactionRequest;

		let network_magic = 1337u64;
		// retrieved test vector from:
		// https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
		let tx: Transaction = TransactionRequest {
			from: None,
			to: Some("F0109fC8DF283027b6285cc889F5aA624EaC1F55".parse::<Address>().unwrap().into()),
			value: Some(1_000_000_000u64.into()),
			gas: Some(2_000_000u64.into()),
			nonce: Some(0u64.into()),
			gas_price: Some(21_000_000_000u128.into()),
			data: None,
			network_magic: None,
		}
		.into();
		let wallet: Wallet = "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
			.parse()
			.unwrap();
		let wallet = wallet.with_network_magic(network_magic);

		// this should populate the tx network_magic as the signer's network_magic (1337) before signing and
		// normalize the v
		let sig = wallet.sign_transaction_sync(&tx).unwrap();

		// ensure correct v given the chain - first extract recid
		let recid = (sig.v - 35) % 2;
		// eip155 check
		assert_eq!(sig.v, network_magic * 2 + 35 + recid);

		// since we initialize with None we need to re-set the network_magic for the sighash to be
		// correct
		let mut tx = tx;
		tx.set_network_magic(network_magic);
		let sighash = tx.sighash();
		sig.verify(sighash, wallet.address).unwrap();
	}

	#[test]
	fn key_to_address() {
		let wallet: Wallet = "0000000000000000000000000000000000000000000000000000000000000001"
			.parse()
			.unwrap();
		assert_eq!(
			wallet.address,
			Address::from_str("7E5F4552091A69125d5DfCb7b8C2659029395Bdf").expect("Decoding failed")
		);

		let wallet: Wallet = "0000000000000000000000000000000000000000000000000000000000000002"
			.parse()
			.unwrap();
		assert_eq!(
			wallet.address,
			Address::from_str("2B5AD5c4795c026514f8317c7a215E218DcCD6cF").expect("Decoding failed")
		);

		let wallet: Wallet = "0000000000000000000000000000000000000000000000000000000000000003"
			.parse()
			.unwrap();
		assert_eq!(
			wallet.address,
			Address::from_str("6813Eb9362372EEF6200f3b1dbC3f819671cBA69").expect("Decoding failed")
		);
	}

	#[test]
	fn key_from_bytes() {
		let wallet: Wallet = "0000000000000000000000000000000000000000000000000000000000000001"
			.parse()
			.unwrap();

		let key_as_bytes = wallet.signer.to_bytes();
		let wallet_from_bytes = Wallet::from_bytes(&key_as_bytes).unwrap();

		assert_eq!(wallet.address, wallet_from_bytes.address);
		assert_eq!(wallet.network_magic, wallet_from_bytes.network_magic);
		assert_eq!(wallet.signer, wallet_from_bytes.signer);
	}

	#[test]
	fn key_from_str() {
		let wallet: Wallet = "0000000000000000000000000000000000000000000000000000000000000001"
			.parse()
			.unwrap();

		// Check FromStr and `0x`
		let wallet_0x: Wallet =
			"0x0000000000000000000000000000000000000000000000000000000000000001"
				.parse()
				.unwrap();
		assert_eq!(wallet.address, wallet_0x.address);
		assert_eq!(wallet.network_magic, wallet_0x.network_magic);
		assert_eq!(wallet.signer, wallet_0x.signer);

		// Check FromStr and `0X`
		let wallet_0x_cap: Wallet =
			"0X0000000000000000000000000000000000000000000000000000000000000001"
				.parse()
				.unwrap();
		assert_eq!(wallet.address, wallet_0x_cap.address);
		assert_eq!(wallet.network_magic, wallet_0x_cap.network_magic);
		assert_eq!(wallet.signer, wallet_0x_cap.signer);

		// Check TryFrom<&str>
		let wallet_0x_tryfrom_str: Wallet =
			"0x0000000000000000000000000000000000000000000000000000000000000001"
				.try_into()
				.unwrap();
		assert_eq!(wallet.address, wallet_0x_tryfrom_str.address);
		assert_eq!(wallet.network_magic, wallet_0x_tryfrom_str.network_magic);
		assert_eq!(wallet.signer, wallet_0x_tryfrom_str.signer);

		// Check TryFrom<String>
		let wallet_0x_tryfrom_string: Wallet =
			"0x0000000000000000000000000000000000000000000000000000000000000001"
				.to_string()
				.try_into()
				.unwrap();
		assert_eq!(wallet.address, wallet_0x_tryfrom_string.address);
		assert_eq!(wallet.network_magic, wallet_0x_tryfrom_string.network_magic);
		assert_eq!(wallet.signer, wallet_0x_tryfrom_string.signer);

		// Must fail because of `0z`
		"0z0000000000000000000000000000000000000000000000000000000000000001"
			.parse::<Wallet>()
			.unwrap_err();
	}
}
