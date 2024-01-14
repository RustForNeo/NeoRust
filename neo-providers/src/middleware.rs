use crate::{
	core::{
		account::AccountTrait,
		responses::{
			neo_address::NeoAddress,
			neo_application_log::ApplicationLog,
			neo_balances::{Nep11Balances, Nep17Balances},
			neo_block::NeoBlock,
			neo_find_states::States,
			neo_get_mem_pool::MemPoolDetails,
			neo_get_next_block_validators::Validator,
			neo_get_peers::Peers,
			neo_get_state_height::StateHeight,
			neo_get_state_root::StateRoot,
			neo_get_unclaimed_gas::UnclaimedGas,
			neo_get_version::NeoVersion,
			neo_get_wallet_balance::Balance,
			neo_list_plugins::Plugin,
			neo_send_raw_transaction::RawTransaction,
			neo_transaction_result::TransactionResult,
			neo_transfers::{Nep11Transfers, Nep17Transfers},
			neo_validate_address::ValidateAddress,
		},
		transaction::{
			signers::signer::Signer, transaction::Transaction,
			transaction_send_token::TransactionSendToken, witness::Witness,
		},
	},
	EscalatingPending, EscalationPolicy, FilterKind, FilterWatcher, JsonRpcClient, LogQuery,
	MiddlewareError, NodeInfo, PeerInfo, PendingTransaction, Provider, ProviderError, PubsubClient,
	SubscriptionStream,
};
use async_trait::async_trait;
use auto_impl::auto_impl;
use neo_config::NeoConfig;
use neo_crypto::keys::Secp256r1Signature;
use neo_types::{
	address::{Address, NameOrAddress},
	block::{Block, BlockId},
	contract_parameter::ContractParameter,
	contract_state::ContractState,
	filter::Filter,
	invocation_result::InvocationResult,
	log::Log,
	native_contract_state::NativeContractState,
	stack_item::StackItem,
	syncing::SyncingStatus,
	Bytes, TxHash,
};
use primitive_types::{H160, H256, U256};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, vec};
use url::Url;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[auto_impl(&, Box, Arc)]
pub trait Middleware: Sync + Send + Debug {
	/// Error type returned by most operations
	type Error: MiddlewareError<Inner = <<Self as Middleware>::Inner as Middleware>::Error>;
	/// The JSON-RPC client type at the bottom of the stack
	type Provider: JsonRpcClient;
	/// The next-lower middleware in the middleware stack
	type Inner: Middleware<Provider = Self::Provider>;

	/// Get a reference to the next-lower middleware in the middleware stack
	fn inner(&self) -> &Self::Inner;

	/// Convert a provider error into the associated error type by successively
	/// converting it to every intermediate middleware error
	fn convert_err(p: ProviderError) -> Self::Error {
		Self::Error::from_provider_err(p)
	}

	/// The HTTP or Websocket provider.
	fn provider(&self) -> &Provider<Self::Provider> {
		self.inner().provider()
	}

	fn config(&self) -> &NeoConfig {
		&self.inner().config()
	}

	/// Return the default sender (if any). This will typically be the
	/// connected node's first address, or the address of a Signer in a lower
	/// middleware stack
	fn default_sender(&self) -> Option<Address> {
		self.inner().default_sender()
	}

	/// Returns the current client version using the `web3_clientVersion` RPC.
	async fn client_version(&self) -> Result<String, Self::Error> {
		self.inner().client_version().await.map_err(MiddlewareError::from_err)
	}

	async fn fill_transaction(&self, tx: &mut Transaction) -> Result<(), Self::Error> {
		self.inner().fill_transaction(tx).await.map_err(MiddlewareError::from_err)
	}

	/// Get the block number
	async fn get_block_number(&self) -> Result<u64, Self::Error> {
		self.inner().get_block_number().await.map_err(MiddlewareError::from_err)
	}

	/// Sends the transaction to the entire Neo network and returns the
	/// transaction's hash. This will consume gas from the account that signed
	/// the transaction. This call will fail if no signer is available, and the
	/// RPC node does  not have an unlocked accounts
	async fn send_transaction<T: Into<Transaction> + Send + Sync>(
		&self,
		tx: T,
	) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
		self.inner().send_transaction(tx).await.map_err(MiddlewareError::from_err)
	}

	////// Neo Naming Service
	// The Neo Naming Service (NNS) allows easy to remember and use names to
	// be assigned to Neo addresses. Any provider operation which takes an address
	// may also take an NNS name.
	//
	// NNS also provides the ability for a reverse lookup, which determines the name for an address
	// if it has been configured.

	/// Returns the address that the `nns_name` resolves to (or None if not configured).
	///
	/// # Panics
	///
	/// If the bytes returned from the NNS registrar/resolver cannot be interpreted as
	/// an address. This should theoretically never happen.
	async fn resolve_name(&self, nns_name: &str) -> Result<Address, Self::Error> {
		self.inner().resolve_name(nns_name).await.map_err(MiddlewareError::from_err)
	}

	/// Returns the NNS name the `address` resolves to (or None if not configured).
	///
	/// # Panics
	///
	/// If the bytes returned from the NNS registrar/resolver cannot be interpreted as
	/// a string. This should theoretically never happen.
	async fn lookup_address(&self, address: Address) -> Result<String, Self::Error> {
		self.inner().lookup_address(address).await.map_err(MiddlewareError::from_err)
	}

	/// Fetch a field for the `nns_name` (no None if not configured).
	///
	/// # Panics
	///
	/// If the bytes returned from the NNS registrar/resolver cannot be interpreted as
	/// a string. This should theoretically never happen.
	async fn resolve_field(&self, nns_name: &str, field: &str) -> Result<String, Self::Error> {
		self.inner()
			.resolve_field(nns_name, field)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// /// Gets the block at `block_hash_or_number` (transaction hashes only)
	// async fn get_block<T: Into<BlockId> + Send + Sync>(
	// 	&self,
	// 	block_hash_or_number: T,
	// ) -> Result<Option<Block<TxHash>>, Self::Error> {
	// 	self.inner()
	// 		.get_block(block_hash_or_number)
	// 		.await
	// 		.map_err(MiddlewareError::from_err)
	// }

	/// Gets the block at `block_hash_or_number` (full transactions included)
	async fn get_block_with_txs<T: Into<BlockId> + Send + Sync>(
		&self,
		block_hash_or_number: T,
	) -> Result<Option<Block<TransactionResult, Witness>>, Self::Error> {
		self.inner()
			.get_block_with_txs(block_hash_or_number)
			.await
			.map_err(MiddlewareError::from_err)
	}

	/// Sends the read-only (constant) transaction to a single Neo node and return the result
	/// (as bytes) of executing it. This is free, since it does not change any state on the
	/// blockchain.
	async fn call(&self, tx: &Transaction, block: Option<BlockId>) -> Result<Bytes, Self::Error> {
		self.inner().call(tx, block).await.map_err(MiddlewareError::from_err)
	}

	/// Return current client syncing status. If IsFalse sync is over.
	async fn syncing(&self) -> Result<SyncingStatus, Self::Error> {
		self.inner().syncing().await.map_err(MiddlewareError::from_err)
	}

	/// Returns the currently configured network magic, a value used in replay-protected
	/// transaction signing as introduced by EIP-155.
	async fn get_network_magic(&self) -> Result<u32, Self::Error> {
		self.inner().get_network_magic().await.map_err(MiddlewareError::from_err)
	}

	/// Returns the network version.
	async fn get_net_version(&self) -> Result<String, Self::Error> {
		self.inner().get_net_version().await.map_err(MiddlewareError::from_err)
	}

	/// Returns the account's balance
	async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
		&self,
		from: T,
		block: Option<BlockId>,
	) -> Result<U256, Self::Error> {
		self.inner().get_balance(from, block).await.map_err(MiddlewareError::from_err)
	}

	/// Gets the transaction with `transaction_hash`
	// async fn get_transaction<T: Send + Sync + Into<TxHash>>(
	// 	&self,
	// 	transaction_hash: T,
	// ) -> Result<Option<Transaction>, Self::Error> {
	// 	self.inner()
	// 		.get_transaction(transaction_hash)
	// 		.await
	// 		.map_err(MiddlewareError::from_err)
	// }

	/// Gets the transaction with block and index
	async fn get_transaction_by_block_and_index<T: Into<BlockId> + Send + Sync>(
		&self,
		block_hash_or_number: T,
		idx: u64,
	) -> Result<Option<TransactionResult>, Self::Error> {
		self.inner()
			.get_transaction_by_block_and_index(block_hash_or_number, idx)
			.await
			.map_err(MiddlewareError::from_err)
	}

	/// Gets the transaction receipt with `transaction_hash`
	async fn get_transaction_receipt<T: Send + Sync + Into<TxHash>>(
		&self,
		transaction_hash: T,
	) -> Result<Option<TransactionResult>, Self::Error> {
		self.inner()
			.get_transaction_receipt(transaction_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	/// Returns all receipts for a block.
	///
	/// Note that this uses the `neo_getBlockReceipts` RPC, which is
	/// non-standard and currently supported by Erigon.
	async fn get_block_receipts<T: Into<BlockId> + Send + Sync>(
		&self,
		block: T,
	) -> Result<Vec<TransactionResult>, Self::Error> {
		self.inner().get_block_receipts(block).await.map_err(MiddlewareError::from_err)
	}

	/// Gets the current gas price as estimated by the node
	async fn get_gas_price(&self) -> Result<U256, Self::Error> {
		self.inner().get_gas_price().await.map_err(MiddlewareError::from_err)
	}

	/// Gets a heuristic recommendation of max fee per gas and max priority fee per gas for
	/// EIP-1559 compatible transactions.
	async fn estimate_eip1559_fees(
		&self,
		estimator: Option<fn(U256, Vec<Vec<U256>>) -> (U256, U256)>,
	) -> Result<(U256, U256), Self::Error> {
		self.inner()
			.estimate_eip1559_fees(estimator)
			.await
			.map_err(MiddlewareError::from_err)
	}

	/// Gets the accounts on the node
	async fn get_accounts(&self) -> Result<Vec<Address>, Self::Error> {
		self.inner().get_accounts().await.map_err(MiddlewareError::from_err)
	}

	/// Send the raw RLP encoded transaction to the entire Neo network and returns the
	/// transaction's hash This will consume gas from the account that signed the transaction.
	// async fn send_raw_transaction<'a>(
	// 	&'a self,
	// 	tx: Bytes,
	// ) -> Result<PendingTransaction<'a, Self::Provider>, Self::Error> {
	// 	self.inner().send_raw_transaction(tx).await.map_err(MiddlewareError::from_err)
	// }

	/// This returns true if either the middleware stack contains a `SignerMiddleware`, or the
	/// JSON-RPC provider has an unlocked key that can sign using the `neo_sign` call. If none of
	/// the above conditions are met, then the middleware stack is not capable of signing data.
	async fn is_signer(&self) -> bool {
		self.inner().is_signer().await
	}

	/// Signs data using a specific account. This account needs to be unlocked,
	/// or the middleware stack must contain a `SignerMiddleware`
	async fn sign<T: Into<Bytes> + Send + Sync>(
		&self,
		data: T,
		from: &Address,
	) -> Result<Secp256r1Signature, Self::Error> {
		self.inner().sign(data, from).await.map_err(MiddlewareError::from_err)
	}

	/// Sign a transaction via RPC call
	async fn sign_transaction(
		&self,
		tx: &Transaction,
		from: Address,
	) -> Result<Secp256r1Signature, Self::Error> {
		self.inner().sign_transaction(tx, from).await.map_err(MiddlewareError::from_err)
	}

	////// Contract state

	/// Returns an array (possibly empty) of logs that match the filter
	async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, Self::Error> {
		self.inner().get_logs(filter).await.map_err(MiddlewareError::from_err)
	}

	/// Returns a stream of logs are loaded in pages of given page size
	fn get_logs_paginated<'a>(
		&'a self,
		filter: &Filter,
		page_size: u64,
	) -> LogQuery<'a, Self::Provider> {
		self.inner().get_logs_paginated(filter, page_size)
	}

	/// Install a new filter on the node.
	///
	/// This method is hidden because filter lifecycle  should be managed by
	/// the [`FilterWatcher`]
	#[doc(hidden)]
	async fn new_filter(&self, filter: FilterKind<'_>) -> Result<U256, Self::Error> {
		self.inner().new_filter(filter).await.map_err(MiddlewareError::from_err)
	}

	/// Uninstalls a filter.
	///
	/// This method is hidden because filter lifecycle  should be managed by
	/// the [`FilterWatcher`]
	#[doc(hidden)]
	async fn uninstall_filter<T: Into<U256> + Send + Sync>(
		&self,
		id: T,
	) -> Result<bool, Self::Error> {
		self.inner().uninstall_filter(id).await.map_err(MiddlewareError::from_err)
	}

	/// Streams event logs matching the filter.
	///
	/// This function streams via a polling system, by repeatedly dispatching
	/// RPC requests. If possible, prefer using a WS or IPC connection and the
	/// `stream` interface
	async fn watch<'a>(
		&'a self,
		filter: &Filter,
	) -> Result<FilterWatcher<'a, Self::Provider, Log>, Self::Error> {
		self.inner().watch(filter).await.map_err(MiddlewareError::from_err)
	}

	/// Streams pending transactions.
	///
	/// This function streams via a polling system, by repeatedly dispatching
	/// RPC requests. If possible, prefer using a WS or IPC connection and the
	/// `stream` interface
	async fn watch_pending_transactions(
		&self,
	) -> Result<FilterWatcher<'_, Self::Provider, H256>, Self::Error> {
		self.inner()
			.watch_pending_transactions()
			.await
			.map_err(MiddlewareError::from_err)
	}

	/// Polling method for a filter, which returns an array of logs which occurred since last poll.
	///
	/// This method must be called with one of the following return types, depending on the filter
	/// type:
	/// - `neo_newBlockFilter`: [`H256`], returns block hashes
	/// - `neo_newPendingTransactionFilter`: [`H256`], returns transaction hashes
	/// - `neo_newFilter`: [`Log`], returns raw logs
	///
	/// If one of these types is not used, decoding will fail and the method will
	/// return an error.
	///
	/// [`H256`]: neo_types::H256
	/// [`Log`]: neo_types::Log
	///
	/// This method is hidden because filter lifecycle  should be managed by
	/// the [`FilterWatcher`]
	#[doc(hidden)]
	async fn get_filter_changes<T, R>(&self, id: T) -> Result<Vec<R>, Self::Error>
	where
		T: Into<U256> + Send + Sync,
		R: Serialize + DeserializeOwned + Send + Sync + Debug,
	{
		self.inner().get_filter_changes(id).await.map_err(MiddlewareError::from_err)
	}

	/// Streams new block hashes
	///
	/// This function streams via a polling system, by repeatedly dispatching
	/// RPC requests. If possible, prefer using a WS or IPC connection and the
	/// `stream` interface
	async fn watch_blocks(&self) -> Result<FilterWatcher<'_, Self::Provider, H256>, Self::Error> {
		self.inner().watch_blocks().await.map_err(MiddlewareError::from_err)
	}

	/// Returns the deployed code at a given address
	async fn get_code<T: Into<NameOrAddress> + Send + Sync>(
		&self,
		at: T,
		block: Option<BlockId>,
	) -> Result<Bytes, Self::Error> {
		self.inner().get_code(at, block).await.map_err(MiddlewareError::from_err)
	}

	/// Get the storage of an address for a particular slot location
	async fn get_storage_at<T: Into<NameOrAddress> + Send + Sync>(
		&self,
		from: T,
		location: H256,
		block: Option<BlockId>,
	) -> Result<H256, Self::Error> {
		self.inner()
			.get_storage_at(from, location, block)
			.await
			.map_err(MiddlewareError::from_err)
	}

	/// Sends the given key to the node to be encrypted with the provided
	/// passphrase and stored.
	///
	/// The key represents a secp256k1 private key and should be 32 bytes.
	async fn import_raw_key(
		&self,
		private_key: Bytes,
		passphrase: String,
	) -> Result<Address, Self::Error> {
		self.inner()
			.import_raw_key(private_key, passphrase)
			.await
			.map_err(MiddlewareError::from_err)
	}

	/// Prompts the node to decrypt the given account from its keystore.
	///
	/// If the duration provided is `None`, then the account will be unlocked
	/// indefinitely. Otherwise, the account will be unlocked for the provided
	/// number of seconds.
	async fn unlock_account<T: Into<Address> + Send + Sync>(
		&self,
		account: T,
		passphrase: String,
		duration: Option<u64>,
	) -> Result<bool, Self::Error> {
		self.inner()
			.unlock_account(account, passphrase, duration)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// Admin namespace

	/// Requests adding the given peer, returning a boolean representing
	/// whether or not the peer was accepted for tracking.
	async fn add_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
		self.inner().add_peer(enode_url).await.map_err(MiddlewareError::from_err)
	}

	/// Requests adding the given peer as a trusted peer, which the node will
	/// always connect to even when its peer slots are full.
	async fn add_trusted_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
		self.inner()
			.add_trusted_peer(enode_url)
			.await
			.map_err(MiddlewareError::from_err)
	}

	/// Returns general information about the node as well as information about the running p2p
	/// protocols (e.g. `eth`, `snap`).
	async fn node_info(&self) -> Result<NodeInfo, Self::Error> {
		self.inner().node_info().await.map_err(MiddlewareError::from_err)
	}

	/// Returns the list of peers currently connected to the node.
	async fn peers(&self) -> Result<Vec<PeerInfo>, Self::Error> {
		self.inner().peers().await.map_err(MiddlewareError::from_err)
	}

	/// Requests to remove the given peer, returning true if the enode was successfully parsed and
	/// the peer was removed.
	async fn remove_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
		self.inner().remove_peer(enode_url).await.map_err(MiddlewareError::from_err)
	}

	/// Requests to remove the given peer, returning a boolean representing whether or not the
	/// enode url passed was validated. A return value of `true` does not necessarily mean that the
	/// peer was disconnected.
	async fn remove_trusted_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
		self.inner()
			.remove_trusted_peer(enode_url)
			.await
			.map_err(MiddlewareError::from_err)
	}

	/// Create a new subscription
	///
	/// This method is hidden as subscription lifecycles are intended to be
	/// handled by a [`SubscriptionStream`] object.
	#[doc(hidden)]
	async fn subscribe<T, R>(
		&self,
		params: T,
	) -> Result<SubscriptionStream<'_, Self::Provider, R>, Self::Error>
	where
		T: Debug + Serialize + Send + Sync,
		R: DeserializeOwned + Send + Sync,
		<Self as Middleware>::Provider: PubsubClient,
	{
		self.inner().subscribe(params).await.map_err(MiddlewareError::from_err)
	}

	/// Instruct the RPC to cancel a subscription by its ID
	///
	/// This method is hidden as subscription lifecycles are intended to be
	/// handled by a [`SubscriptionStream`] object
	#[doc(hidden)]
	async fn unsubscribe<T>(&self, id: T) -> Result<bool, Self::Error>
	where
		T: Into<U256> + Send + Sync,
		<Self as Middleware>::Provider: PubsubClient,
	{
		self.inner().unsubscribe(id).await.map_err(MiddlewareError::from_err)
	}

	/// Subscribe to a stream of incoming blocks.
	///
	/// This function is only available on pubsub clients, such as Websockets
	/// or IPC. For a polling alternative available over HTTP, use
	/// [`Middleware::watch_blocks`]. However, be aware that polling increases
	/// RPC usage drastically.
	async fn subscribe_blocks(
		&self,
	) -> Result<SubscriptionStream<'_, Self::Provider, Block<TxHash, Witness>>, Self::Error>
	where
		<Self as Middleware>::Provider: PubsubClient,
	{
		self.inner().subscribe_blocks().await.map_err(MiddlewareError::from_err)
	}

	/// Subscribe to a stream of pending transaction hashes.
	///
	/// This function is only available on pubsub clients, such as Websockets
	/// or IPC. For a polling alternative available over HTTP, use
	/// [`Middleware::watch_pending_transactions`]. However, be aware that
	/// polling increases RPC usage drastically.
	async fn subscribe_pending_txs(
		&self,
	) -> Result<SubscriptionStream<'_, Self::Provider, TxHash>, Self::Error>
	where
		<Self as Middleware>::Provider: PubsubClient,
	{
		self.inner().subscribe_pending_txs().await.map_err(MiddlewareError::from_err)
	}

	/// Subscribe to a stream of pending transaction bodies.
	///
	/// This function is only available on pubsub clients, such as Websockets
	/// or IPC. For a polling alternative available over HTTP, use
	/// [`Middleware::watch_pending_transactions`]. However, be aware that
	/// polling increases RPC usage drastically.
	///
	/// Note: This endpoint is compatible only with Geth client version 1.11.0 or later.
	async fn subscribe_full_pending_txs(
		&self,
	) -> Result<SubscriptionStream<'_, Self::Provider, Transaction>, Self::Error>
	where
		<Self as Middleware>::Provider: PubsubClient,
	{
		self.inner()
			.subscribe_full_pending_txs()
			.await
			.map_err(MiddlewareError::from_err)
	}

	/// Subscribe to a stream of event logs matchin the provided [`Filter`].
	///
	/// This function is only available on pubsub clients, such as Websockets
	/// or IPC. For a polling alternative available over HTTP, use
	/// [`Middleware::watch`]. However, be aware that polling increases
	/// RPC usage drastically.
	async fn subscribe_logs<'a>(
		&'a self,
		filter: &Filter,
	) -> Result<SubscriptionStream<'a, Self::Provider, Log>, Self::Error>
	where
		<Self as Middleware>::Provider: PubsubClient,
	{
		self.inner().subscribe_logs(filter).await.map_err(MiddlewareError::from_err)
	}

	fn nns_resolver(&self) -> H160 {
		H160::from(self.config().nns_resolver.clone())
	}

	fn block_interval(&self) -> u32 {
		self.config().block_interval
	}

	fn polling_interval(&self) -> u32 {
		self.config().polling_interval
	}

	fn max_valid_until_block_increment(&self) -> u32 {
		self.config().max_valid_until_block_increment
	}

	async fn dump_private_key(&self, script_hash: H160) -> Result<String, Self::Error> {
		self.inner()
			.dump_private_key(script_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// async fn get_network_magic_number(&self) -> Result<u32, Self::Error> {
	// 	self.inner().get_network_magic_number().await.map_err(MiddlewareError::from_err)
	// }

	// async fn get_network_magic_number_bytes(&self) -> Result<Bytes, Self::Error> {
	// 	self.inner()
	// 		.get_network_magic_number_bytes()
	// 		.await
	// 		.map_err(MiddlewareError::from_err)
	// }

	// Blockchain methods
	async fn get_best_block_hash(&self) -> Result<H256, Self::Error> {
		self.inner().get_best_block_hash().await.map_err(MiddlewareError::from_err)
	}

	async fn get_block_hash(&self, block_index: u32) -> Result<H256, Self::Error> {
		self.inner()
			.get_block_hash(block_index)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_block(&self, block_hash: H256, full_tx: bool) -> Result<NeoBlock, Self::Error> {
		self.inner()
			.get_block(block_hash, full_tx)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_raw_block(&self, block_hash: H256) -> Result<String, Self::Error> {
		self.inner().get_raw_block(block_hash).await.map_err(MiddlewareError::from_err)
	}

	// Node methods

	async fn get_block_header_count(&self) -> Result<u32, Self::Error> {
		self.inner().get_block_count().await.map_err(MiddlewareError::from_err)
	}

	async fn get_block_count(&self) -> Result<u32, Self::Error> {
		self.inner().get_block_count().await.map_err(MiddlewareError::from_err)
	}

	async fn get_block_header(&self, block_hash: H256) -> Result<NeoBlock, Self::Error> {
		self.inner()
			.get_block_header(block_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_block_header_by_index(&self, index: u32) -> Result<NeoBlock, Self::Error> {
		self.inner()
			.get_block_header_by_index(index)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// Smart contract methods

	async fn get_raw_block_header(&self, block_hash: H256) -> Result<String, Self::Error> {
		self.inner()
			.get_raw_block_header(block_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_raw_block_header_by_index(&self, index: u32) -> Result<String, Self::Error> {
		self.inner()
			.get_raw_block_header_by_index(index)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// Utility methods

	async fn get_native_contracts(&self) -> Result<Vec<NativeContractState>, Self::Error> {
		self.inner().get_native_contracts().await.map_err(MiddlewareError::from_err)
	}

	// Wallet methods

	async fn get_contract_state(&self, hash: H160) -> Result<ContractState, Self::Error> {
		self.inner().get_contract_state(hash).await.map_err(MiddlewareError::from_err)
	}

	async fn get_native_contract_state(&self, name: &str) -> Result<ContractState, Self::Error> {
		self.inner()
			.get_native_contract_state(name)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_mem_pool(&self) -> Result<MemPoolDetails, Self::Error> {
		self.inner().get_mem_pool().await.map_err(MiddlewareError::from_err)
	}

	async fn get_raw_mem_pool(&self) -> Result<Vec<H256>, Self::Error> {
		self.inner().get_raw_mem_pool().await.map_err(MiddlewareError::from_err)
	}

	// Application logs

	async fn get_transaction(&self, hash: H256) -> Result<Option<TransactionResult>, Self::Error> {
		self.inner().get_transaction(hash).await.map_err(MiddlewareError::from_err)
	}

	// State service

	async fn get_raw_transaction(&self, tx_hash: H256) -> Result<RawTransaction, Self::Error> {
		self.inner()
			.get_raw_transaction(tx_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_storage(&self, contract_hash: H160, key: &str) -> Result<String, Self::Error> {
		self.inner()
			.get_storage(contract_hash, key)
			.await
			.map_err(MiddlewareError::from_err)
	}
	// Blockchain methods

	async fn get_transaction_height(&self, tx_hash: H256) -> Result<u32, Self::Error> {
		self.inner()
			.get_transaction_height(tx_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_next_block_validators(&self) -> Result<Vec<Validator>, Self::Error> {
		self.inner()
			.get_next_block_validators()
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_committee(&self) -> Result<Vec<String>, Self::Error> {
		self.inner().get_committee().await.map_err(MiddlewareError::from_err)
	}

	async fn get_connection_count(&self) -> Result<u32, Self::Error> {
		self.inner().get_connection_count().await.map_err(MiddlewareError::from_err)
	}

	async fn get_peers(&self) -> Result<Peers, Self::Error> {
		self.inner().get_peers().await.map_err(MiddlewareError::from_err)
	}

	// Smart contract method
	async fn get_version(&self) -> Result<NeoVersion, Self::Error> {
		self.inner().get_version().await.map_err(MiddlewareError::from_err)
	}

	async fn send_raw_transaction(&self, hex: String) -> Result<RawTransaction, Self::Error> {
		self.inner().send_raw_transaction(hex).await.map_err(MiddlewareError::from_err)
	}

	async fn submit_block(&self, hex: String) -> Result<bool, Self::Error> {
		self.inner().submit_block(hex).await.map_err(MiddlewareError::from_err)
	}

	// Blockchain methods
	async fn invoke_function<T: AccountTrait + Serialize>(
		&self,
		contract_hash: &H160,
		method: String,
		params: Vec<ContractParameter>,
		signers: Option<Vec<Signer<T>>>,
	) -> Result<InvocationResult, Self::Error> {
		self.inner()
			.invoke_function(contract_hash, method, params, signers)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn invoke_script<T: AccountTrait + Serialize>(
		&self,
		hex: String,
		signers: Vec<Signer<T>>,
	) -> Result<InvocationResult, Self::Error> {
		self.inner()
			.invoke_script(hex, signers)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// More smart contract methods

	async fn get_unclaimed_gas(&self, hash: H160) -> Result<UnclaimedGas, Self::Error> {
		self.inner().get_unclaimed_gas(hash).await.map_err(MiddlewareError::from_err)
	}

	async fn list_plugins(&self) -> Result<Vec<Plugin>, Self::Error> {
		self.inner().list_plugins().await.map_err(MiddlewareError::from_err)
	}

	// Utility methods

	async fn validate_address(&self, address: &str) -> Result<ValidateAddress, Self::Error> {
		self.inner().validate_address(address).await.map_err(MiddlewareError::from_err)
	}

	// Wallet methods

	async fn close_wallet(&self) -> Result<bool, Self::Error> {
		self.inner().close_wallet().await.map_err(MiddlewareError::from_err)
	}

	async fn dump_priv_key(&self, script_hash: H160) -> Result<String, Self::Error> {
		self.inner().dump_priv_key(script_hash).await.map_err(MiddlewareError::from_err)
	}

	async fn get_wallet_balance(&self, token_hash: H160) -> Result<Balance, Self::Error> {
		self.inner()
			.get_wallet_balance(token_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_new_address(&self) -> Result<String, Self::Error> {
		self.inner().get_new_address().await.map_err(MiddlewareError::from_err)
	}

	async fn get_wallet_unclaimed_gas(&self) -> Result<String, Self::Error> {
		self.inner().get_wallet_unclaimed_gas().await.map_err(MiddlewareError::from_err)
	}

	async fn import_priv_key(&self, priv_key: String) -> Result<NeoAddress, Self::Error> {
		self.inner().import_priv_key(priv_key).await.map_err(MiddlewareError::from_err)
	}

	async fn calculate_network_fee(&self, hex: String) -> Result<u64, Self::Error> {
		self.inner().calculate_network_fee(hex).await.map_err(MiddlewareError::from_err)
	}

	async fn list_address(&self) -> Result<Vec<NeoAddress>, Self::Error> {
		self.inner().list_address().await.map_err(MiddlewareError::from_err)
	}
	async fn open_wallet(&self, path: String, password: String) -> Result<bool, Self::Error> {
		self.inner()
			.open_wallet(path, password)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn send_from(
		&self,
		token_hash: H160,
		from: Address,
		to: Address,
		amount: u32,
	) -> Result<Transaction, Self::Error> {
		self.inner()
			.send_from(token_hash, from, to, amount)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// Transaction methods

	async fn send_many(
		&self,
		from: Option<H160>,
		send_tokens: Vec<TransactionSendToken>,
	) -> Result<Transaction, Self::Error> {
		self.inner()
			.send_many(from, send_tokens)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn send_to_address(
		&self,
		token_hash: H160,
		to: Address,
		amount: u32,
	) -> Result<Transaction, Self::Error> {
		self.inner()
			.send_to_address(token_hash, to, amount)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_application_log(&self, tx_hash: H256) -> Result<ApplicationLog, Self::Error> {
		self.inner()
			.get_application_log(tx_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_nep17_balances(&self, script_hash: H160) -> Result<Nep17Balances, Self::Error> {
		self.inner()
			.get_nep17_balances(script_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_nep17_transfers(&self, script_hash: H160) -> Result<Nep17Transfers, Self::Error> {
		self.inner()
			.get_nep17_transfers(script_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// NEP-17 methods

	async fn get_nep17_transfers_from(
		&self,
		script_hash: H160,
		from: u64,
	) -> Result<Nep17Transfers, Self::Error> {
		self.inner()
			.get_nep17_transfers_from(script_hash, from)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_nep17_transfers_range(
		&self,
		script_hash: H160,
		from: u64,
		to: u64,
	) -> Result<Nep17Transfers, Self::Error> {
		self.inner()
			.get_nep17_transfers_range(script_hash, from, to)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_nep11_balances(&self, script_hash: H160) -> Result<Nep11Balances, Self::Error> {
		self.inner()
			.get_nep11_balances(script_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// NEP-11 methods

	async fn get_nep11_transfers(&self, script_hash: H160) -> Result<Nep11Transfers, Self::Error> {
		self.inner()
			.get_nep11_transfers(script_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_nep11_transfers_from(
		&self,
		script_hash: H160,
		from: u64,
	) -> Result<Nep11Transfers, Self::Error> {
		self.inner()
			.get_nep11_transfers_from(script_hash, from)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_nep11_transfers_range(
		&self,
		script_hash: H160,
		from: u64,
		to: u64,
	) -> Result<Nep11Transfers, Self::Error> {
		self.inner()
			.get_nep11_transfers_range(script_hash, from, to)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_nep11_properties(
		&self,
		script_hash: H160,
		token_id: &str,
	) -> Result<HashMap<String, String>, Self::Error> {
		self.inner()
			.get_nep11_properties(script_hash, token_id)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_state_root(&self, block_index: u32) -> Result<StateRoot, Self::Error> {
		self.inner()
			.get_state_root(block_index)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// State service methods
	async fn get_proof(
		&self,
		root_hash: H256,
		contract_hash: H160,
		key: &str,
	) -> Result<String, Self::Error> {
		self.inner()
			.get_proof(root_hash, contract_hash, key)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn verify_proof(&self, root_hash: H256, proof: &str) -> Result<bool, Self::Error> {
		self.inner()
			.verify_proof(root_hash, proof)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_state_height(&self) -> Result<StateHeight, Self::Error> {
		self.inner().get_state_height().await.map_err(MiddlewareError::from_err)
	}

	async fn get_state(
		&self,
		root_hash: H256,
		contract_hash: H160,
		key: &str,
	) -> Result<String, Self::Error> {
		self.inner()
			.get_state(root_hash, contract_hash, key)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn find_states(
		&self,
		root_hash: H256,
		contract_hash: H160,
		key_prefix: &str,
		start_key: Option<&str>,
		count: Option<u32>,
	) -> Result<States, Self::Error> {
		self.inner()
			.find_states(root_hash, contract_hash, key_prefix, start_key, count)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_block_by_index(&self, index: u32, full_tx: bool) -> Result<NeoBlock, Self::Error> {
		self.inner()
			.get_block_by_index(index, full_tx)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_raw_block_by_index(&self, index: u32) -> Result<String, Self::Error> {
		self.inner()
			.get_raw_block_by_index(index)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn invoke_function_diagnostics<T: AccountTrait + Serialize>(
		&self,
		contract_hash: H160,
		name: String,
		params: Vec<ContractParameter>,
		signers: Vec<Signer<T>>,
	) -> Result<InvocationResult, Self::Error> {
		self.inner()
			.invoke_function_diagnostics(contract_hash, name, params, signers)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn invoke_script_diagnostics<T: AccountTrait + Serialize>(
		&self,
		hex: String,
		signers: Vec<Signer<T>>,
	) -> Result<InvocationResult, Self::Error> {
		self.inner()
			.invoke_script_diagnostics(hex, signers)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn traverse_iterator(
		&self,
		session_id: String,
		iterator_id: String,
		count: u32,
	) -> Result<Vec<StackItem>, Self::Error> {
		self.inner()
			.traverse_iterator(session_id, iterator_id, count)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn terminate_session(&self, session_id: &str) -> Result<bool, Self::Error> {
		self.inner()
			.terminate_session(session_id)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn invoke_contract_verify<T: AccountTrait + Serialize>(
		&self,
		hash: H160,
		params: Vec<ContractParameter>,
		signers: Vec<Signer<T>>,
	) -> Result<InvocationResult, Self::Error> {
		self.inner()
			.invoke_contract_verify(hash, params, signers)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_raw_mempool(&self) -> Result<MemPoolDetails, Self::Error> {
		self.inner().get_raw_mempool().await.map_err(MiddlewareError::from_err)
	}

	async fn import_private_key(&self, wif: String) -> Result<NeoAddress, Self::Error> {
		self.inner().import_private_key(wif).await.map_err(MiddlewareError::from_err)
	}

	async fn get_block_header_hash(&self, hash: H256) -> Result<NeoBlock, Self::Error> {
		self.inner()
			.get_block_header_hash(hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn send_to_address_send_token(
		&self,
		send_token: &TransactionSendToken,
	) -> Result<Transaction, Self::Error> {
		self.inner()
			.send_to_address_send_token(send_token)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn send_from_send_token(
		&self,
		send_token: &TransactionSendToken,
		from: Address,
	) -> Result<Transaction, Self::Error> {
		self.inner()
			.send_from_send_token(send_token, from)
			.await
			.map_err(MiddlewareError::from_err)
	}
}
