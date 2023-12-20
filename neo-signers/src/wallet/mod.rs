mod mnemonic;
pub use mnemonic::{MnemonicBuilder, MnemonicBuilderError};

mod private_key;
pub use private_key::WalletError;

pub mod account;
mod nep6account;
mod nep6contract;
mod nep6wallet;
mod wallet;
pub(crate) mod wallet_error;
#[cfg(all(feature = "yubihsm", not(target_arch = "wasm32")))]
mod yubi;

use crate::Signer;

use crate::transaction::transaction::Transaction;
use async_trait::async_trait;
use neo_crypto::signature::Signature;
use neo_types::address::Address;
use p256::ecdsa::signature::hazmat::PrehashSigner;
use primitive_types::{H256, U256};
use std::fmt;

/// A neo private-public key pair which can be used for signing messages.
///
/// # Examples
///
/// ## Signing and Verifying a message
///
/// The wallet can be used to produce ECDSA [`Signature`] objects, which can be
/// then verified. Note that this uses [`hash_message`] under the hood which will
/// prefix the message being hashed with the `neo Signed Message` domain separator.
///
/// ```
/// use neo_core::rand::thread_rng;
/// use neo_signers::{LocalWallet, Signer};
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// let wallet = LocalWallet::new(&mut thread_rng());
///
/// // Optionally, the wallet's network magic can be set, in order to use
/// // replay protection with different chains
/// let wallet = wallet.with_network_magic(1337u64);
///
/// // The wallet can be used to sign messages
/// let message = b"hello";
/// let signature = wallet.sign_message(message).await?;
/// assert_eq!(signature.recover(&message[..]).unwrap(), wallet.address());
///
/// // LocalWallet is clonable:
/// let wallet_clone = wallet.clone();
/// let signature2 = wallet_clone.sign_message(message).await?;
/// assert_eq!(signature, signature2);
/// # Ok(())
/// # }
/// ```
///
/// [`Signature`]: neo_types::Signature
/// [`hash_message`]: fn@neo_core::utils::hash_message
#[derive(Clone)]
pub struct Wallet<D: PrehashSigner<Signature>> {
	/// The Wallet's private Key
	pub(crate) signer: D,
	/// The wallet's address
	pub(crate) address: Address,
	/// The wallet's network magic (for EIP-155)
	pub(crate) network_magic: u64,
}

impl<D: PrehashSigner<Signature>> Wallet<D> {
	/// Construct a new wallet with an external Signer
	pub fn new_with_signer(signer: D, address: Address, network_magic: u64) -> Self {
		Wallet { signer, address, network_magic }
	}
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<D: Sync + Send + PrehashSigner<Signature>> Signer for Wallet<D> {
	type Error = WalletError;

	async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
		&self,
		message: S,
	) -> Result<Signature, Self::Error> {
		let message = message.as_ref();
		let message_hash = hash_message(message);

		self.sign_hash(message_hash)
	}

	async fn sign_transaction(&self, tx: &Transaction) -> Result<Signature, Self::Error> {
		let mut tx_with_chain = tx.clone();
		if tx_with_chain.network_magic().is_none() {
			// in the case we don't have a network_magic, let's use the signer network magic instead
			tx_with_chain.set_network_magic(self.network_magic);
		}
		self.sign_transaction_sync(&tx_with_chain)
	}

	fn address(&self) -> Address {
		self.address
	}

	/// Gets the wallet's network magic
	fn network_magic(&self) -> u64 {
		self.network_magic
	}

	/// Sets the wallet's network_magic, used in conjunction with EIP-155 signing
	fn with_network_magic<T: Into<u64>>(mut self, network_magic: T) -> Self {
		self.network_magic = network_magic.into();
		self
	}
}

impl<D: PrehashSigner<Signature>> Wallet<D> {
	/// Synchronously signs the provided transaction, normalizing the signature `v` value with
	/// EIP-155 using the transaction's `network_magic`, or the signer's `network_magic` if the transaction
	/// does not specify one.
	pub fn sign_transaction_sync(&self, tx: &Transaction) -> Result<Signature, WalletError> {
		// rlp (for sighash) must have the same network magic as v in the signature
		let network_magic = tx.network_magic().map(|id| id.as_u64()).unwrap_or(self.network_magic);
		let mut tx = tx.clone();
		tx.set_network_magic(network_magic);

		let sighash = tx.sighash();
		let mut sig = self.sign_hash(sighash)?;

		// sign_hash sets `v` to recid + 27, so we need to subtract 27 before normalizing
		sig.v = to_eip155_v(sig.v as u8 - 27, network_magic);
		Ok(sig)
	}

	/// Signs the provided hash.
	pub fn sign_hash(&self, hash: H256) -> Result<Signature, WalletError> {
		let (recoverable_sig, recovery_id) = self.signer.sign_prehash(hash.as_ref())?;

		let v = u8::from(recovery_id) as u64 + 27;

		let r_bytes: FieldBytes<Secp256r1> = recoverable_sig.r().into();
		let s_bytes: FieldBytes<Secp256r1> = recoverable_sig.s().into();
		let r = U256::from_big_endian(r_bytes.as_slice());
		let s = U256::from_big_endian(s_bytes.as_slice());

		Ok(Signature { r, s, v })
	}

	/// Gets the wallet's signer
	pub fn signer(&self) -> &D {
		&self.signer
	}
}

// do not log the signer
impl<D: PrehashSigner<Signature>> fmt::Debug for Wallet<D> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Wallet")
			.field("address", &self.address)
			.field("chain_Id", &self.network_magic)
			.finish()
	}
}
