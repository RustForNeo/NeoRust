pub mod app;
pub mod types;

use crate::Signer;
use app::Ledgerneo;
use async_trait::async_trait;
use neo_crypto::signature::Signature;
use neo_types::address::Address;
use types::LedgerError;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Signer for Ledgerneo {
	type Error = LedgerError;

	/// Signs the hash of the provided message after prefixing it
	async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
		&self,
		message: S,
	) -> Result<Signature, Self::Error> {
		self.sign_message(message).await
	}

	/// Signs the transaction
	async fn sign_transaction(&self, message: &TypedTransaction) -> Result<Signature, Self::Error> {
		let mut tx_with_chain = message.clone();
		if tx_with_chain.network_magic().is_none() {
			// in the case we don't have a network_magic, let's use the signer network magic instead
			tx_with_chain.set_network_magic(self.network_magic);
		}
		self.sign_tx(&tx_with_chain).await
	}

	/// Returns the signer's neo Address
	fn address(&self) -> Address {
		self.address
	}

	fn network_magic(&self) -> u64 {
		self.network_magic
	}

	fn with_network_magic<T: Into<u64>>(mut self, network_magic: T) -> Self {
		self.network_magic = network_magic.into();
		self
	}
}
