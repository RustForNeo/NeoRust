use crate::{
	call_raw::CallBuilder,
	errors::ProviderError,
	ext::nns,
	rpc::pubsub::{PubsubClient, SubscriptionStream},
	stream::{FilterWatcher, DEFAULT_LOCAL_POLL_INTERVAL, DEFAULT_POLL_INTERVAL},
	utils,
	utils::maybe,
	Http as HttpProvider, JsonRpcClient, JsonRpcClientWrapper, LogQuery, MiddlewareError,
	MockProvider, NodeInfo, PeerInfo, PendingTransaction, QuorumProvider, RwClient,
};

pub use crate::Middleware;
#[cfg(not(target_arch = "wasm32"))]
use crate::{HttpRateLimitRetryPolicy, RetryClient};
use std::net::Ipv4Addr;

use async_trait::async_trait;

use futures_util::{lock::Mutex, try_join};
use neo_types::{
	address::Address,
	block::{Block, BlockId},
	contract_parameter::ContractParameter,
	filter::Filter,
	invocation_result::InvocationResult,
};
use primitive_types::{H160, H256 as TxHash, U256};
use serde::{de::DeserializeOwned, Serialize};
use std::{
	collections::VecDeque, convert::TryFrom, fmt::Debug, str::FromStr, sync::Arc, time::Duration,
};
use tracing::trace;
use tracing_futures::Instrument;
use url::{Host, ParseError, Url};

/// Node Clients
#[derive(Copy, Clone)]
pub enum NodeClient {
	/// Geth
	Geth,
}

impl FromStr for NodeClient {
	type Err = ProviderError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.split('/').next().unwrap().to_lowercase().as_str() {
			"geth" => Ok(NodeClient::Geth),
			_ => Err(ProviderError::UnsupportedNodeClient),
		}
	}
}

/// An abstract provider for interacting with the [Neo JSON RPC
/// API](https://github.com/neo/wiki/wiki/JSON-RPC). Must be instantiated
/// with a data transport which implements the [`JsonRpcClient`](trait@crate::JsonRpcClient) trait
/// (e.g. [HTTP](crate::Http), Websockets etc.)
///
/// # Example
///
/// ```no_run
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// use neo_providers::{Middleware, Provider, Http};
/// use std::convert::TryFrom;
///
/// let provider = Provider::<Http>::try_from(
///     "https://eth.llamarpc.com"
/// ).expect("could not instantiate HTTP Provider");
///
/// let block = provider.get_block(100u64).await?;
/// println!("Got block: {}", serde_json::to_string(&block)?);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Provider<P> {
	inner: P,
	nns: Option<Address>,
	interval: Option<Duration>,
	from: Option<Address>,
	_node_client: Arc<Mutex<Option<NodeClient>>>,
}

impl<P> AsRef<P> for Provider<P> {
	fn as_ref(&self) -> &P {
		&self.inner
	}
}

/// Types of filters supported by the JSON-RPC.
#[derive(Clone, Debug)]
pub enum FilterKind<'a> {
	/// `neo_newBlockFilter`
	Logs(&'a Filter),

	/// `neo_newBlockFilter` filter
	NewBlocks,

	/// `neo_newPendingTransactionFilter` filter
	PendingTransactions,
}

// JSON RPC bindings
impl<P: JsonRpcClient> Provider<P> {
	/// Instantiate a new provider with a backend.
	pub fn new(provider: P) -> Self {
		Self {
			inner: provider,
			nns: None,
			interval: None,
			from: None,
			_node_client: Arc::new(Mutex::new(None)),
		}
	}

	/// Returns the type of node we're connected to, while also caching the value for use
	/// in other node-specific API calls, such as the get_block_receipts call.
	pub async fn node_client(&self) -> Result<NodeClient, ProviderError> {
		let mut node_client = self._node_client.lock().await;

		if let Some(node_client) = *node_client {
			Ok(node_client)
		} else {
			let client_version = self.client_version().await?;
			let client_version = match client_version.parse::<NodeClient>() {
				Ok(res) => res,
				Err(_) => return Err(ProviderError::UnsupportedNodeClient),
			};
			*node_client = Some(client_version);
			Ok(client_version)
		}
	}

	#[must_use]
	/// Set the default sender on the provider
	pub fn with_sender(mut self, address: impl Into<Address>) -> Self {
		self.from = Some(address.into());
		self
	}

	/// Make an RPC request via the internal connection, and return the result.
	pub async fn request<T, R>(&self, method: &str, params: T) -> Result<R, ProviderError>
	where
		T: Debug + Serialize + Send + Sync,
		R: Serialize + DeserializeOwned + Debug + Send,
	{
		let span =
			tracing::trace_span!("rpc", method = method, params = ?serde_json::to_string(&params)?);
		// https://docs.rs/tracing/0.1.22/tracing/span/struct.Span.html#in-asynchronous-code
		let res = async move {
			trace!("tx");
			let res: R = self.inner.request(method, params).await.map_err(Into::into)?;
			trace!(rx = ?serde_json::to_string(&res)?);
			Ok::<_, ProviderError>(res)
		}
		.instrument(span)
		.await?;
		Ok(res)
	}

	async fn get_block_gen<Tx: Default + Serialize + DeserializeOwned + Debug + Send>(
		&self,
		id: BlockId,
		include_txs: bool,
	) -> Result<Option<Block<Tx>>, ProviderError> {
		let include_txs = utils::serialize(&include_txs);

		Ok(match id {
			BlockId::Hash(hash) => {
				let hash = utils::serialize(&hash);
				self.request("neo_getBlockByHash", [hash, include_txs]).await?
			},
			BlockId::Number(num) => {
				let num = utils::serialize(&num);
				self.request("neo_getBlockByNumber", [num, include_txs]).await?
			},
		})
	}

	/// Analogous to [`Middleware::call`], but returns a [`CallBuilder`] that can either be
	/// `.await`d or used to override the parameters sent to `neo_call`.
	///
	/// See the [`neo_types::spoof`] for functions to construct state override
	/// parameters.
	///
	/// Note: this method _does not_ send a transaction from your account
	///
	/// [`neo_types::spoof`]: neo_types::spoof
	///
	/// # Example
	///
	/// ```no_run
	/// # use neo_core::{
	/// #     types::{Address, TransactionRequest, H256, spoof},
	/// #     utils::{parse_ether, Geth},
	/// # };
	/// # use neo_providers::{Provider, Http, Middleware, call_raw::RawCall};
	/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
	/// let geth = Geth::new().spawn();
	/// let provider = Provider::<Http>::try_from(geth.endpoint()).unwrap();
	///
	/// let adr1: Address = "0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse()?;
	/// let adr2: Address = "0x295a70b2de5e3953354a6a8344e616ed314d7251".parse()?;
	/// let pay_amt = parse_ether(1u64)?;
	///
	/// // Not enough ether to pay for the transaction
	/// let tx = TransactionRequest::pay(adr2, pay_amt).from(adr1).into();
	///
	/// // override the sender's balance for the call
	/// let mut state = spoof::balance(adr1, pay_amt * 2);
	/// provider.call_raw(&tx).state(&state).await?;
	/// # Ok(()) }
	/// ```
	pub fn call_raw<'a>(&'a self, tx: &'a Transaction) -> CallBuilder<'a, P> {
		CallBuilder::new(self, tx)
	}
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<P: JsonRpcClient> Middleware for Provider<P> {
	type Error = ProviderError;
	type Provider = P;
	type Inner = Self;

	fn inner(&self) -> &Self::Inner {
		unreachable!("There is no inner provider here")
	}

	fn provider(&self) -> &Provider<Self::Provider> {
		self
	}

	fn convert_err(p: ProviderError) -> Self::Error {
		// no conversion necessary
		p
	}

	fn default_sender(&self) -> Option<Address> {
		self.from
	}

	async fn client_version(&self) -> Result<String, Self::Error> {
		self.request("web3_clientVersion", ()).await
	}

	async fn fill_transaction(
		&self,
		tx: &mut Transaction,
		block: Option<BlockId>,
	) -> Result<(), Self::Error> {
		if let Some(default_sender) = self.default_sender() {
			if tx.from().is_none() {
				tx.set_from(default_sender);
			}
		}

		// TODO: Join the name resolution and gas price future

		// set the NNS name
		if let Some(NameOrAddress::Name(ref nns_name)) = tx.to() {
			let addr = self.resolve_name(nns_name).await?;
			tx.set_to(addr);
		}

		// Set gas to estimated value only if it was not set by the caller,
		// even if the access list has been populated and saves gas
		if tx.gas().is_none() {
			let gas_estimate = self.estimate_gas(tx, block).await?;
			tx.set_gas(gas_estimate);
		}

		Ok(())
	}

	async fn get_block_number(&self) -> Result<U64, ProviderError> {
		self.request("neo_blockNumber", ()).await
	}

	async fn get_block<T: Into<BlockId> + Send + Sync>(
		&self,
		block_hash_or_number: T,
	) -> Result<Option<Block>, Self::Error> {
		self.get_block_gen(block_hash_or_number.into(), false).await
	}

	async fn get_block_with_txs<T: Into<BlockId> + Send + Sync>(
		&self,
		block_hash_or_number: T,
	) -> Result<Option<Block<Transaction>>, ProviderError> {
		self.get_block_gen(block_hash_or_number.into(), true).await
	}

	async fn get_transaction<T: Send + Sync + Into<TxHash>>(
		&self,
		transaction_hash: T,
	) -> Result<Option<Transaction>, ProviderError> {
		let hash = transaction_hash.into();
		self.request("neo_getTransactionByHash", [hash]).await
	}

	async fn get_transaction_by_block_and_index<T: Into<BlockId> + Send + Sync>(
		&self,
		block_hash_or_number: T,
		idx: U64,
	) -> Result<Option<Transaction>, ProviderError> {
		let blk_id = block_hash_or_number.into();
		let idx = utils::serialize(&idx);
		Ok(match blk_id {
			BlockId::Hash(hash) => {
				let hash = utils::serialize(&hash);
				self.request("neo_getTransactionByBlockHashAndIndex", [hash, idx]).await?
			},
			BlockId::Number(num) => {
				let num = utils::serialize(&num);
				self.request("neo_getTransactionByBlockNumberAndIndex", [num, idx]).await?
			},
		})
	}

	async fn get_transaction_receipt<T: Send + Sync + Into<TxHash>>(
		&self,
		transaction_hash: T,
	) -> Result<Option<TransactionReceipt>, ProviderError> {
		let hash = transaction_hash.into();
		self.request("neo_getTransactionReceipt", [hash]).await
	}

	async fn get_block_receipts<T: Into<BlockNumber> + Send + Sync>(
		&self,
		block: T,
	) -> Result<Vec<TransactionReceipt>, Self::Error> {
		self.request("neo_getBlockReceipts", [block.into()]).await
	}

	async fn parity_block_receipts<T: Into<BlockNumber> + Send + Sync>(
		&self,
		block: T,
	) -> Result<Vec<TransactionReceipt>, Self::Error> {
		self.request("parity_getBlockReceipts", vec![block.into()]).await
	}

	async fn get_gas_price(&self) -> Result<U256, ProviderError> {
		self.request("neo_gasPrice", ()).await
	}

	async fn estimate_eip1559_fees(
		&self,
		estimator: Option<fn(U256, Vec<Vec<U256>>) -> (U256, U256)>,
	) -> Result<(U256, U256), Self::Error> {
		let base_fee_per_gas = self
			.get_block(BlockNumber::Latest)
			.await?
			.ok_or_else(|| ProviderError::CustomError("Latest block not found".into()))?
			.base_fee_per_gas
			.ok_or_else(|| ProviderError::CustomError("EIP-1559 not activated".into()))?;

		let fee_history = self
			.fee_history(
				utils::EIP1559_FEE_ESTIMATION_PAST_BLOCKS,
				BlockNumber::Latest,
				&[utils::EIP1559_FEE_ESTIMATION_REWARD_PERCENTILE],
			)
			.await?;

		// use the provided fee estimator function, or fallback to the default implementation.
		let (max_fee_per_gas, max_priority_fee_per_gas) = if let Some(es) = estimator {
			es(base_fee_per_gas, fee_history.reward)
		} else {
			utils::eip1559_default_estimator(base_fee_per_gas, fee_history.reward)
		};

		Ok((max_fee_per_gas, max_priority_fee_per_gas))
	}

	async fn get_accounts(&self) -> Result<Vec<Address>, ProviderError> {
		self.request("neo_accounts", ()).await
	}

	async fn get_transaction_count<T: Into<NameOrAddress> + Send + Sync>(
		&self,
		from: T,
		block: Option<BlockId>,
	) -> Result<U256, ProviderError> {
		let from = match from.into() {
			NameOrAddress::Name(nns_name) => self.resolve_name(&nns_name).await?,
			NameOrAddress::Address(addr) => addr,
		};

		let from = utils::serialize(&from);
		let block = utils::serialize(&block.unwrap_or_else(|| BlockNumber::Latest.into()));
		self.request("neo_getTransactionCount", [from, block]).await
	}

	async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
		&self,
		from: T,
		block: Option<BlockId>,
	) -> Result<U256, ProviderError> {
		let from = match from.into() {
			NameOrAddress::Name(nns_name) => self.resolve_name(&nns_name).await?,
			NameOrAddress::Address(addr) => addr,
		};

		let from = utils::serialize(&from);
		let block = utils::serialize(&block.unwrap_or_else(|| BlockNumber::Latest.into()));
		self.request("neo_getBalance", [from, block]).await
	}

	async fn get_network_magic(&self) -> Result<U256, ProviderError> {
		self.request("neo_chainId", ()).await
	}

	async fn syncing(&self) -> Result<SyncingStatus, Self::Error> {
		self.request("neo_syncing", ()).await
	}

	async fn get_net_version(&self) -> Result<String, ProviderError> {
		self.request("net_version", ()).await
	}

	async fn call(&self, tx: &Transaction, block: Option<BlockId>) -> Result<Bytes, ProviderError> {
		let tx = utils::serialize(tx);
		let block = utils::serialize(&block.unwrap_or_else(|| BlockNumber::Latest.into()));
		self.request("neo_call", [tx, block]).await
	}

	async fn estimate_gas(
		&self,
		tx: &Transaction,
		block: Option<BlockId>,
	) -> Result<U256, ProviderError> {
		let tx = utils::serialize(tx);
		// Some nodes (e.g. old Optimism clients) don't support a block ID being passed as a param,
		// so refrain from defaulting to BlockNumber::Latest.
		let params = if let Some(block_id) = block {
			vec![tx, utils::serialize(&block_id)]
		} else {
			vec![tx]
		};
		self.request("neo_estimateGas", params).await
	}

	async fn create_access_list(
		&self,
		tx: &Transaction,
		block: Option<BlockId>,
	) -> Result<AccessListWithGasUsed, ProviderError> {
		let tx = utils::serialize(tx);
		let block = utils::serialize(&block.unwrap_or_else(|| BlockNumber::Latest.into()));
		self.request("neo_createAccessList", [tx, block]).await
	}

	async fn send_transaction<T: Into<Transaction> + Send + Sync>(
		&self,
		tx: T,
		block: Option<BlockId>,
	) -> Result<PendingTransaction<'_, P>, ProviderError> {
		let mut tx = tx.into();
		self.fill_transaction(&mut tx, block).await?;
		let tx_hash = self.request("neo_sendTransaction", [tx]).await?;

		Ok(PendingTransaction::new(tx_hash, self))
	}

	async fn send_raw_transaction<'a>(
		&'a self,
		tx: Bytes,
	) -> Result<PendingTransaction<'a, P>, ProviderError> {
		let rlp = utils::serialize(&tx);
		let tx_hash = self.request("neo_sendRawTransaction", [rlp]).await?;
		Ok(PendingTransaction::new(tx_hash, self))
	}

	async fn is_signer(&self) -> bool {
		match self.from {
			Some(sender) => self.sign(vec![], &sender).await.is_ok(),
			None => false,
		}
	}

	async fn sign<T: Into<Bytes> + Send + Sync>(
		&self,
		data: T,
		from: &Address,
	) -> Result<Signature, ProviderError> {
		let data = utils::serialize(&data.into());
		let from = utils::serialize(from);

		// get the response from `neo_sign` call
		let sig: String = self.request("neo_sign", [from, data]).await?;

		// decode the signature
		let sig = hex::decode(sig)?;
		Ok(Signature::try_from(sig.as_slice())
			.map_err(|e| ProviderError::CustomError(e.to_string()))?)
	}

	/// Sign a transaction via RPC call
	async fn sign_transaction(
		&self,
		_tx: &Transaction,
		_from: Address,
	) -> Result<Signature, Self::Error> {
		Err(MiddlewareError::from_err(ProviderError::SignerUnavailable))
	}

	////// Contract state

	async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, ProviderError> {
		self.request("neo_getLogs", [filter]).await
	}

	fn get_logs_paginated<'a>(&'a self, filter: &Filter, page_size: u64) -> LogQuery<'a, P> {
		LogQuery::new(self, filter).with_page_size(page_size)
	}

	async fn watch<'a>(
		&'a self,
		filter: &Filter,
	) -> Result<FilterWatcher<'a, P, Log>, ProviderError> {
		let id = self.new_filter(FilterKind::Logs(filter)).await?;
		let filter = FilterWatcher::new(id, self).interval(self.get_interval());
		Ok(filter)
	}

	async fn watch_blocks(&self) -> Result<FilterWatcher<'_, P, H256>, ProviderError> {
		let id = self.new_filter(FilterKind::NewBlocks).await?;
		let filter = FilterWatcher::new(id, self).interval(self.get_interval());
		Ok(filter)
	}

	/// Streams pending transactions
	async fn watch_pending_transactions(
		&self,
	) -> Result<FilterWatcher<'_, P, H256>, ProviderError> {
		let id = self.new_filter(FilterKind::PendingTransactions).await?;
		let filter = FilterWatcher::new(id, self).interval(self.get_interval());
		Ok(filter)
	}

	async fn new_filter(&self, filter: FilterKind<'_>) -> Result<U256, ProviderError> {
		let (method, args) = match filter {
			FilterKind::NewBlocks => ("neo_newBlockFilter", vec![]),
			FilterKind::PendingTransactions => ("neo_newPendingTransactionFilter", vec![]),
			FilterKind::Logs(filter) => ("neo_newFilter", vec![utils::serialize(&filter)]),
		};

		self.request(method, args).await
	}

	async fn uninstall_filter<T: Into<U256> + Send + Sync>(
		&self,
		id: T,
	) -> Result<bool, ProviderError> {
		let id = utils::serialize(&id.into());
		self.request("neo_uninstallFilter", [id]).await
	}

	async fn get_filter_changes<T, R>(&self, id: T) -> Result<Vec<R>, ProviderError>
	where
		T: Into<U256> + Send + Sync,
		R: Serialize + DeserializeOwned + Send + Sync + Debug,
	{
		let id = utils::serialize(&id.into());
		self.request("neo_getFilterChanges", [id]).await
	}

	async fn get_storage_at<T: Into<NameOrAddress> + Send + Sync>(
		&self,
		from: T,
		location: H256,
		block: Option<BlockId>,
	) -> Result<H256, ProviderError> {
		let from = match from.into() {
			NameOrAddress::Name(nns_name) => self.resolve_name(&nns_name).await?,
			NameOrAddress::Address(addr) => addr,
		};

		// position is a QUANTITY according to the [spec](https://eth.wiki/json-rpc/API#neo_getstorageat): integer of the position in the storage, converting this to a U256
		// will make sure the number is formatted correctly as [quantity](https://eips.neo.org/EIPS/eip-1474#quantity)
		let position = U256::from_big_endian(location.as_bytes());
		let position = utils::serialize(&position);
		let from = utils::serialize(&from);
		let block = utils::serialize(&block.unwrap_or_else(|| BlockNumber::Latest.into()));

		// get the hex encoded value
		let value: String = self.request("neo_getStorageAt", [from, position, block]).await?;
		// decode and left-pad to 32 bytes
		let bytes = hex::decode(value)?;
		if bytes.len() > 32 {
			Err(hex::FromHexError::InvalidStringLength.into())
		} else {
			let mut buf = [0; 32];
			buf[32 - bytes.len()..].copy_from_slice(&bytes);
			Ok(H256(buf))
		}
	}

	async fn get_code<T: Into<NameOrAddress> + Send + Sync>(
		&self,
		at: T,
		block: Option<BlockId>,
	) -> Result<Bytes, ProviderError> {
		let at = match at.into() {
			NameOrAddress::Name(nns_name) => self.resolve_name(&nns_name).await?,
			NameOrAddress::Address(addr) => addr,
		};

		let at = utils::serialize(&at);
		let block = utils::serialize(&block.unwrap_or_else(|| BlockNumber::Latest.into()));
		self.request("neo_getCode", [at, block]).await
	}

	async fn import_raw_key(
		&self,
		private_key: Bytes,
		passphrase: String,
	) -> Result<Address, ProviderError> {
		// private key should not be prefixed with 0x - it is also up to the user to pass in a key
		// of the correct length

		// the private key argument is supposed to be a string
		let private_key_hex = hex::encode(private_key);
		let private_key = utils::serialize(&private_key_hex);
		let passphrase = utils::serialize(&passphrase);
		self.request("personal_importRawKey", [private_key, passphrase]).await
	}

	async fn unlock_account<T: Into<Address> + Send + Sync>(
		&self,
		account: T,
		passphrase: String,
		duration: Option<u64>,
	) -> Result<bool, ProviderError> {
		let account = utils::serialize(&account.into());
		let duration = utils::serialize(&duration.unwrap_or(0));
		let passphrase = utils::serialize(&passphrase);
		self.request("personal_unlockAccount", [account, passphrase, duration]).await
	}

	async fn add_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
		let enode_url = utils::serialize(&enode_url);
		self.request("admin_addPeer", [enode_url]).await
	}

	async fn add_trusted_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
		let enode_url = utils::serialize(&enode_url);
		self.request("admin_addTrustedPeer", [enode_url]).await
	}

	async fn node_info(&self) -> Result<NodeInfo, Self::Error> {
		self.request("admin_nodeInfo", ()).await
	}

	async fn peers(&self) -> Result<Vec<PeerInfo>, Self::Error> {
		self.request("admin_peers", ()).await
	}

	async fn remove_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
		let enode_url = utils::serialize(&enode_url);
		self.request("admin_removePeer", [enode_url]).await
	}

	async fn remove_trusted_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
		let enode_url = utils::serialize(&enode_url);
		self.request("admin_removeTrustedPeer", [enode_url]).await
	}

	async fn start_mining(&self) -> Result<(), Self::Error> {
		self.request("miner_start", ()).await
	}

	async fn stop_mining(&self) -> Result<(), Self::Error> {
		self.request("miner_stop", ()).await
	}

	async fn resolve_name(&self, nns_name: &str) -> Result<Address, ProviderError> {
		self.query_resolver(ParamType::Address, nns_name, nns::ADDR_SELECTOR).await
	}

	async fn lookup_address(&self, address: Address) -> Result<String, ProviderError> {
		let nns_name = nns::reverse_address(address);
		let domain: String =
			self.query_resolver(ParamType::String, &nns_name, nns::NAME_SELECTOR).await?;
		let reverse_address = self.resolve_name(&domain).await?;
		if address != reverse_address {
			Err(ProviderError::NnsNotOwned(domain))
		} else {
			Ok(domain)
		}
	}

	async fn resolve_avatar(&self, nns_name: &str) -> Result<Url, ProviderError> {
		let (field, owner) =
			try_join!(self.resolve_field(nns_name, "avatar"), self.resolve_name(nns_name))?;
		let url = Url::from_str(&field).map_err(|e| ProviderError::CustomError(e.to_string()))?;
		match url.scheme() {
			"https" | "data" => Ok(url),
			"ipfs" => erc::http_link_ipfs(url).map_err(ProviderError::CustomError),
			"eip155" => {
				let token =
					erc::ERCNFT::from_str(url.path()).map_err(ProviderError::CustomError)?;
				match token.type_ {
					erc::ERCNFTType::ERC721 => {
						let tx = TransactionRequest {
							data: Some(
								[&erc::ERC721_OWNER_SELECTOR[..], &token.id].concat().into(),
							),
							to: Some(NameOrAddress::Address(token.contract)),
							..Default::default()
						};
						let data = self.call(&tx.into(), None).await?;
						if decode_bytes::<Address>(ParamType::Address, data) != owner {
							return Err(ProviderError::CustomError("Incorrect owner.".to_string()))
						}
					},
					erc::ERCNFTType::ERC1155 => {
						let tx = TransactionRequest {
							data: Some(
								[
									&erc::ERC1155_BALANCE_SELECTOR[..],
									&[0x0; 12],
									&owner.0,
									&token.id,
								]
								.concat()
								.into(),
							),
							to: Some(NameOrAddress::Address(token.contract)),
							..Default::default()
						};
						let data = self.call(&tx.into(), None).await?;
						if decode_bytes::<u64>(ParamType::Uint(64), data) == 0 {
							return Err(ProviderError::CustomError("Incorrect balance.".to_string()))
						}
					},
				}

				let image_url = self.resolve_nft(token).await?;
				match image_url.scheme() {
					"https" | "data" => Ok(image_url),
					"ipfs" => erc::http_link_ipfs(image_url).map_err(ProviderError::CustomError),
					_ => Err(ProviderError::CustomError(
						"Unsupported scheme for the image".to_string(),
					)),
				}
			},
			_ => Err(ProviderError::CustomError("Unsupported scheme".to_string())),
		}
	}

	async fn resolve_nft(&self, token: erc::ERCNFT) -> Result<Url, ProviderError> {
		let selector = token.type_.resolution_selector();
		let tx = TransactionRequest {
			data: Some([&selector[..], &token.id].concat().into()),
			to: Some(NameOrAddress::Address(token.contract)),
			..Default::default()
		};
		let data = self.call(&tx.into(), None).await?;
		let mut metadata_url = Url::parse(&decode_bytes::<String>(ParamType::String, data))
			.map_err(|e| ProviderError::CustomError(format!("Invalid metadata url: {e}")))?;

		if token.type_ == erc::ERCNFTType::ERC1155 {
			metadata_url.set_path(&metadata_url.path().replace("%7Bid%7D", &hex::encode(token.id)));
		}
		if metadata_url.scheme() == "ipfs" {
			metadata_url = erc::http_link_ipfs(metadata_url).map_err(ProviderError::CustomError)?;
		}
		let metadata: erc::Metadata = reqwest::get(metadata_url).await?.json().await?;
		Url::parse(&metadata.image).map_err(|e| ProviderError::CustomError(e.to_string()))
	}

	async fn resolve_field(&self, nns_name: &str, field: &str) -> Result<String, ProviderError> {
		let field: String = self
			.query_resolver_parameters(
				ParamType::String,
				nns_name,
				nns::FIELD_SELECTOR,
				Some(&nns::parameterhash(field)),
			)
			.await?;
		Ok(field)
	}

	async fn txpool_content(&self) -> Result<TxpoolContent, ProviderError> {
		self.request("txpool_content", ()).await
	}

	async fn txpool_inspect(&self) -> Result<TxpoolInspect, ProviderError> {
		self.request("txpool_inspect", ()).await
	}

	async fn txpool_status(&self) -> Result<TxpoolStatus, ProviderError> {
		self.request("txpool_status", ()).await
	}

	async fn debug_trace_transaction(
		&self,
		tx_hash: TxHash,
		trace_options: GethDebugTracingOptions,
	) -> Result<GethTrace, ProviderError> {
		let tx_hash = utils::serialize(&tx_hash);
		let trace_options = utils::serialize(&trace_options);
		self.request("debug_traceTransaction", [tx_hash, trace_options]).await
	}

	async fn debug_trace_call<T: Into<Transaction> + Send + Sync>(
		&self,
		req: T,
		block: Option<BlockId>,
		trace_options: GethDebugTracingCallOptions,
	) -> Result<GethTrace, ProviderError> {
		let req = req.into();
		let req = utils::serialize(&req);
		let block = utils::serialize(&block.unwrap_or_else(|| BlockNumber::Latest.into()));
		let trace_options = utils::serialize(&trace_options);
		self.request("debug_traceCall", [req, block, trace_options]).await
	}

	async fn debug_trace_block_by_number(
		&self,
		block: Option<BlockNumber>,
		trace_options: GethDebugTracingOptions,
	) -> Result<Vec<GethTrace>, ProviderError> {
		let block = utils::serialize(&block.unwrap_or(BlockNumber::Latest));
		let trace_options = utils::serialize(&trace_options);
		self.request("debug_traceBlockByNumber", [block, trace_options]).await
	}

	async fn debug_trace_block_by_hash(
		&self,
		block: H256,
		trace_options: GethDebugTracingOptions,
	) -> Result<Vec<GethTrace>, ProviderError> {
		let block = utils::serialize(&block);
		let trace_options = utils::serialize(&trace_options);
		self.request("debug_traceBlockByHash", [block, trace_options]).await
	}

	async fn trace_call<T: Into<Transaction> + Send + Sync>(
		&self,
		req: T,
		trace_type: Vec<TraceType>,
		block: Option<BlockNumber>,
	) -> Result<BlockTrace, ProviderError> {
		let req = req.into();
		let req = utils::serialize(&req);
		let block = utils::serialize(&block.unwrap_or(BlockNumber::Latest));
		let trace_type = utils::serialize(&trace_type);
		self.request("trace_call", [req, trace_type, block]).await
	}

	async fn trace_call_many<T: Into<Transaction> + Send + Sync>(
		&self,
		req: Vec<(T, Vec<TraceType>)>,
		block: Option<BlockNumber>,
	) -> Result<Vec<BlockTrace>, ProviderError> {
		let req: Vec<(Transaction, Vec<TraceType>)> =
			req.into_iter().map(|(tx, trace_type)| (tx.into(), trace_type)).collect();
		let req = utils::serialize(&req);
		let block = utils::serialize(&block.unwrap_or(BlockNumber::Latest));
		self.request("trace_callMany", [req, block]).await
	}

	async fn trace_raw_transaction(
		&self,
		data: Bytes,
		trace_type: Vec<TraceType>,
	) -> Result<BlockTrace, ProviderError> {
		let data = utils::serialize(&data);
		let trace_type = utils::serialize(&trace_type);
		self.request("trace_rawTransaction", [data, trace_type]).await
	}

	async fn trace_replay_transaction(
		&self,
		hash: H256,
		trace_type: Vec<TraceType>,
	) -> Result<BlockTrace, ProviderError> {
		let hash = utils::serialize(&hash);
		let trace_type = utils::serialize(&trace_type);
		self.request("trace_replayTransaction", [hash, trace_type]).await
	}

	async fn trace_replay_block_transactions(
		&self,
		block: BlockNumber,
		trace_type: Vec<TraceType>,
	) -> Result<Vec<BlockTrace>, ProviderError> {
		let block = utils::serialize(&block);
		let trace_type = utils::serialize(&trace_type);
		self.request("trace_replayBlockTransactions", [block, trace_type]).await
	}

	async fn trace_block(&self, block: BlockNumber) -> Result<Vec<Trace>, ProviderError> {
		let block = utils::serialize(&block);
		self.request("trace_block", [block]).await
	}

	async fn trace_filter(&self, filter: TraceFilter) -> Result<Vec<Trace>, ProviderError> {
		let filter = utils::serialize(&filter);
		self.request("trace_filter", vec![filter]).await
	}

	async fn trace_get<T: Into<U64> + Send + Sync>(
		&self,
		hash: H256,
		index: Vec<T>,
	) -> Result<Trace, ProviderError> {
		let hash = utils::serialize(&hash);
		let index: Vec<U64> = index.into_iter().map(|i| i.into()).collect();
		let index = utils::serialize(&index);
		self.request("trace_get", vec![hash, index]).await
	}

	async fn trace_transaction(&self, hash: H256) -> Result<Vec<Trace>, ProviderError> {
		let hash = utils::serialize(&hash);
		self.request("trace_transaction", vec![hash]).await
	}

	async fn subscribe<T, R>(
		&self,
		params: T,
	) -> Result<SubscriptionStream<'_, P, R>, ProviderError>
	where
		T: Debug + Serialize + Send + Sync,
		R: DeserializeOwned + Send + Sync,
		P: PubsubClient,
	{
		let id: U256 = self.request("neo_subscribe", params).await?;
		SubscriptionStream::new(id, self).map_err(Into::into)
	}

	async fn unsubscribe<T>(&self, id: T) -> Result<bool, ProviderError>
	where
		T: Into<U256> + Send + Sync,
		P: PubsubClient,
	{
		self.request("neo_unsubscribe", [id.into()]).await
	}

	async fn subscribe_blocks(
		&self,
	) -> Result<SubscriptionStream<'_, P, Block<TxHash>>, ProviderError>
	where
		P: PubsubClient,
	{
		self.subscribe(["newHeads"]).await
	}

	async fn subscribe_pending_txs(
		&self,
	) -> Result<SubscriptionStream<'_, P, TxHash>, ProviderError>
	where
		P: PubsubClient,
	{
		self.subscribe(["newPendingTransactions"]).await
	}

	async fn subscribe_full_pending_txs(
		&self,
	) -> Result<SubscriptionStream<'_, P, Transaction>, ProviderError>
	where
		P: PubsubClient,
	{
		self.subscribe([utils::serialize(&"newPendingTransactions"), utils::serialize(&true)])
			.await
	}

	async fn subscribe_logs<'a>(
		&'a self,
		filter: &Filter,
	) -> Result<SubscriptionStream<'a, P, Log>, ProviderError>
	where
		P: PubsubClient,
	{
		let loaded_logs = match filter.block_option {
			FilterBlockOption::Range { from_block, to_block: _ } =>
				if from_block.is_none() {
					vec![]
				} else {
					self.get_logs(filter).await?
				},
			FilterBlockOption::AtBlockHash(_block_hash) => self.get_logs(filter).await?,
		};
		let loaded_logs = VecDeque::from(loaded_logs);

		let logs = utils::serialize(&"logs"); // TODO: Make this a static
		let filter = utils::serialize(filter);
		self.subscribe([logs, filter]).await.map(|mut stream| {
			stream.set_loaded_elements(loaded_logs);
			stream
		})
	}

	async fn fee_history<T: Into<U256> + Send + Sync>(
		&self,
		block_count: T,
		last_block: BlockNumber,
		reward_percentiles: &[f64],
	) -> Result<FeeHistory, Self::Error> {
		let block_count = block_count.into();
		let last_block = utils::serialize(&last_block);
		let reward_percentiles = utils::serialize(&reward_percentiles);

		// The blockCount param is expected to be an unsigned integer up to geth v1.10.6.
		// Geth v1.10.7 onwards, this has been updated to a hex encoded form. Failure to
		// decode the param from client side would fallback to the old API spec.
		match self
			.request::<_, FeeHistory>(
				"neo_feeHistory",
				[utils::serialize(&block_count), last_block.clone(), reward_percentiles.clone()],
			)
			.await
		{
			success @ Ok(_) => success,
			err @ Err(_) => {
				let fallback = self
					.request::<_, FeeHistory>(
						"neo_feeHistory",
						[utils::serialize(&block_count.as_u64()), last_block, reward_percentiles],
					)
					.await;

				if fallback.is_err() {
					// if the older fallback also resulted in an error, we return the error from the
					// initial attempt
					return err
				}
				fallback
			},
		}
	}

	//////////////////////// Neo methods////////////////////////////

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

	fn dump_private_key(&self, script_hash: H160) -> NeoRequest<String> {
		NeoRequest::new("dumpprivkey", vec![Value::String(script_hash.to_address())])
	}

	async fn get_network_magic_number(&mut self) -> Result<u32, NeoError> {
		if self.config().network_magic.is_none() {
			let magic = self
				.get_version()
				.request()
				.await
				.unwrap()
				.protocol
				.ok_or(NeoError::IllegalState(
					"Unable to read Network Magic Number from Version".to_string(),
				))
				.unwrap()
				.network;
			self.config.network_magic = Some(magic);
		}
		Ok(self.config().network_magic.unwrap())
	}

	async fn get_network_magic_number_bytes(&mut self) -> Result<Bytes, NeoError> {
		let magic_int = self.get_network_magic_number().await.unwrap() & 0xFFFF_FFFF;
		Ok(magic_int.to_be_bytes().to_vec())
	}

	// Blockchain methods
	fn get_best_block_hash(&self) -> NeoRequest<H256Def> {
		NeoRequest::new("getbestblockhash", vec![])
	}

	fn get_block_hash(&self, block_index: u32) -> NeoRequest<H256Def> {
		NeoRequest::new("getblockhash", [block_index.to_value()].to_vec())
	}

	fn get_block(&self, block_hash: H256, full_tx: bool) -> NeoRequest<NeoBlock> {
		if full_tx {
			NeoRequest::new("getblock", [block_hash.to_value(), 1.to_value()].to_vec())
		} else {
			self.get_block_header_hash(block_hash)
		}
	}

	// More methods...

	fn get_raw_block(&self, block_hash: H256) -> NeoRequest<String> {
		NeoRequest::new("getblock", vec![block_hash.to_value(), 0.to_value()])
	}

	// Node methods

	fn get_block_header_count(&self) -> NeoRequest<u32> {
		NeoRequest::new("getblockheadercount", vec![])
	}

	fn get_block_count(&self) -> NeoRequest<u32> {
		NeoRequest::new("getblockcount", vec![])
	}

	fn get_block_header(&self, block_hash: H256) -> NeoRequest<NeoBlock> {
		NeoRequest::new("getblockheader", vec![block_hash.to_value(), 1.to_value()])
	}

	fn get_block_header_by_index(&self, index: u32) -> NeoRequest<NeoBlock> {
		NeoRequest::new("getblockheader", vec![index.to_value(), 1.to_value()])
	}

	// Smart contract methods

	fn get_raw_block_header(&self, block_hash: H256) -> NeoRequest<String> {
		NeoRequest::new("getblockheader", vec![block_hash.to_value(), 0.to_value()])
	}

	fn get_raw_block_header_by_index(&self, index: u32) -> NeoRequest<String> {
		NeoRequest::new("getblockheader", vec![index.to_value(), 0.to_value()])
	}

	// Utility methods

	fn get_native_contracts(&self) -> NeoRequest<Vec<NativeContractState>> {
		NeoRequest::new("getnativecontracts", vec![])
	}

	// Wallet methods

	fn get_contract_state(&self, hash: H160) -> NeoRequest<ContractState> {
		NeoRequest::new("getcontractstate", vec![hash.to_value()])
	}

	fn get_native_contract_state(&self, name: &str) -> NeoRequest<ContractState> {
		NeoRequest::new("getcontractstate", vec![name.to_value()])
	}

	fn get_mem_pool(&self) -> NeoRequest<MemPoolDetails> {
		NeoRequest::new("getrawmempool", vec![1.to_value()])
	}

	fn get_raw_mem_pool(&self) -> NeoRequest<Vec<H256Def>> {
		NeoRequest::new("getrawmempool", vec![])
	}

	// Application logs

	fn get_transaction(&self, hash: H256) -> NeoRequest<Transaction> {
		NeoRequest::new("getrawtransaction", vec![hash.to_value(), 1.to_value()])
	}

	// State service

	fn get_raw_transaction(&self, tx_hash: H256) -> NeoRequest<String> {
		NeoRequest::new("getrawtransaction", vec![tx_hash.to_value(), 0.to_value()])
	}

	fn get_storage(&self, contract_hash: H160, key: &str) -> NeoRequest<String> {
		let params = [contract_hash.to_value(), key.to_value()];
		NeoRequest::new("getstorage", params.to_vec())
	}
	// Blockchain methods

	fn get_transaction_height(&self, tx_hash: H256) -> NeoRequest<u32> {
		let params = [tx_hash.to_value()];
		NeoRequest::new("gettransactionheight", params.to_vec())
	}

	fn get_next_block_validators(&self) -> NeoRequest<Vec<Validator>> {
		NeoRequest::new("getnextblockvalidators", vec![])
	}

	fn get_committee(&self) -> NeoRequest<Vec<String>> {
		NeoRequest::new("getcommittee", vec![])
	}

	fn get_connection_count(&self) -> NeoRequest<u32> {
		NeoRequest::new("getconnectioncount", vec![])
	}

	fn get_peers(&self) -> NeoRequest<Peers> {
		NeoRequest::new("getpeers", vec![])
	}

	// Smart contract methods

	fn get_version(&self) -> NeoRequest<NeoVersion> {
		NeoRequest::new("getversion", vec![])
	}

	fn send_raw_transaction(&self, hex: String) -> NeoRequest<RawTransaction> {
		NeoRequest::new("sendrawtransaction", vec![hex.to_value()])
	}
	// More node methods

	fn submit_block(&self, hex: String) -> NeoRequest<bool> {
		NeoRequest::new("submitblock", vec![hex.to_value()])
	}

	// More blockchain methods

	fn invoke_function(
		&self,
		contract_hash: &H160,
		method: String,
		params: Vec<ContractParameter>,
		signers: Vec<Signer>,
	) -> NeoRequest<InvocationResult> {
		let signers: Vec<TransactionSigner> = signers.iter().map(|f| f.into()).collect();
		NeoRequest::new(
			"invokefunction",
			vec![
				contract_hash.to_value(),
				method.to_value(),
				params.to_value(),
				signers.to_value(),
			],
		)
	}

	fn invoke_script(&self, hex: String, signers: Vec<Signer>) -> NeoRequest<InvocationResult> {
		let signers: Vec<TransactionSigner> =
			signers.into_iter().map(|signer| signer.into()).collect::<Vec<_>>();
		NeoRequest::new("invokescript", vec![hex.to_value(), signers.to_value()])
	}

	// More smart contract methods

	fn get_unclaimed_gas(&self, hash: H160) -> NeoRequest<GetUnclaimedGas> {
		NeoRequest::new("getunclaimedgas", vec![hash.to_value()])
	}

	fn list_plugins(&self) -> NeoRequest<Vec<Plugin>> {
		NeoRequest::new("listplugins", vec![])
	}

	// More utility methods

	fn validate_address(&self, address: &str) -> NeoRequest<ValidateAddress> {
		NeoRequest::new("validateaddress", vec![address.to_value()])
	}

	// More wallet methods

	fn close_wallet(&self) -> NeoRequest<bool> {
		NeoRequest::new("closewallet", vec![])
	}

	fn dump_priv_key(&self, script_hash: H160) -> NeoRequest<String> {
		let params = [script_hash.to_value()].to_vec();
		NeoRequest::new("dumpprivkey", params)
	}

	fn get_wallet_balance(&self, token_hash: H160) -> NeoRequest<Balance> {
		NeoRequest::new("getwalletbalance", vec![token_hash.to_value()])
	}

	fn get_new_address(&self) -> NeoRequest<String> {
		NeoRequest::new("getnewaddress", vec![])
	}

	fn get_wallet_unclaimed_gas(&self) -> NeoRequest<String> {
		NeoRequest::new("getwalletunclaimedgas", vec![])
	}

	fn import_priv_key(&self, priv_key: String) -> NeoRequest<NeoAddress> {
		let params = [priv_key.to_value()].to_vec();
		NeoRequest::new("importprivkey", params)
	}

	fn calculate_network_fee(&self, hex: String) -> NeoRequest<u64> {
		NeoRequest::new("calculatenetworkfee", vec![hex.to_value()])
	}

	fn list_address(&self) -> NeoRequest<Vec<NeoAddress>> {
		NeoRequest::new("listaddress", vec![])
	}
	fn open_wallet(&self, path: String, password: String) -> NeoRequest<bool> {
		NeoRequest::new("openwallet", vec![path.to_value(), password.to_value()])
	}

	fn send_from(
		&self,
		token_hash: H160,
		from: Address,
		to: Address,
		amount: u32,
	) -> NeoRequest<Transaction> {
		let params =
			[token_hash.to_value(), from.to_value(), to.to_value(), amount.to_value()].to_vec();
		NeoRequest::new("sendfrom", params)
	}

	// Transaction methods

	fn send_many(
		&self,
		from: Option<H160>,
		send_tokens: Vec<TransactionSendToken>,
	) -> NeoRequest<Transaction> {
		let params = [from.unwrap().to_value(), send_tokens.to_value()].to_vec();
		NeoRequest::new("sendmany", params)
	}

	fn send_to_address(
		&self,
		token_hash: H160,
		to: Address,
		amount: u32,
	) -> NeoRequest<Transaction> {
		let params = [token_hash.to_value(), to.to_value(), amount.to_value()].to_vec();
		NeoRequest::new("sendtoaddress", params)
	}

	fn get_application_log(&self, tx_hash: H256) -> NeoRequest<NeoApplicationLog> {
		NeoRequest::new("getapplicationlog", vec![tx_hash.to_value()])
	}

	fn get_nep17_balances(&self, script_hash: H160) -> NeoRequest<Nep17Balances> {
		NeoRequest::new("getnep17balances", [script_hash.to_value()].to_vec())
	}

	fn get_nep17_transfers(&self, script_hash: H160) -> NeoRequest<Nep17Transfers> {
		let params = [script_hash.to_value()].to_vec();
		NeoRequest::new("getnep17transfers", params)
	}

	// NEP-17 methods

	fn get_nep17_transfers_from(&self, script_hash: H160, from: u64) -> NeoRequest<Nep17Transfers> {
		let params = [script_hash.to_value(), from.to_value()].to_vec();
		NeoRequest::new("getnep17transfers", params)
	}

	fn get_nep17_transfers_range(
		&self,
		script_hash: H160,
		from: u64,
		to: u64,
	) -> NeoRequest<Nep17Transfers> {
		let params = [script_hash.to_value(), from.to_value(), to.to_value()].to_vec();
		NeoRequest::new("getnep17transfers", params)
	}

	fn get_nep11_balances(&self, script_hash: H160) -> NeoRequest<Nep11Balances> {
		let params = [script_hash.to_value()].to_vec();
		NeoRequest::new("getnep11balances", params)
	}

	// NEP-11 methods

	fn get_nep11_transfers(&self, script_hash: H160) -> NeoRequest<Nep11Transfers> {
		let params = [script_hash.to_value()].to_vec();
		NeoRequest::new("getnep11transfers", params)
	}

	fn get_nep11_transfers_from(&self, script_hash: H160, from: u64) -> NeoRequest<Nep11Transfers> {
		let params = [script_hash.to_value(), from.to_value()].to_vec();
		NeoRequest::new("getnep11transfers", params)
	}

	fn get_nep11_transfers_range(
		&self,
		script_hash: H160,
		from: u64,
		to: u64,
	) -> NeoRequest<Nep11Transfers> {
		let params = [script_hash.to_value(), from.to_value(), to.to_value()].to_vec();
		NeoRequest::new("getnep11transfers", params)
	}

	fn get_nep11_properties(
		&self,
		script_hash: H160,
		token_id: &str,
	) -> NeoRequest<HashMap<String, String>> {
		let params = [script_hash.to_value(), token_id.to_value()].to_vec();
		NeoRequest::new("getnep11properties", params)
	}

	fn get_state_root(&self, block_index: u32) -> NeoRequest<StateRoot> {
		let params = [block_index.to_value()].to_vec();
		NeoRequest::new("getstateroot", params)
	}

	// State service methods

	fn get_proof(&self, root_hash: H256, contract_hash: H160, key: &str) -> NeoRequest<String> {
		NeoRequest::new(
			"getproof",
			vec![root_hash.to_value(), contract_hash.to_value(), key.to_value()],
		)
	}

	fn verify_proof(&self, root_hash: H256, proof: &str) -> NeoRequest<bool> {
		let params = [root_hash.to_value(), proof.to_value()].to_vec();
		NeoRequest::new("verifyproof", params)
	}

	fn get_state_height(&self) -> NeoRequest<StateHeight> {
		NeoRequest::new("getstateheight", vec![])
	}

	fn get_state(&self, root_hash: H256, contract_hash: H160, key: &str) -> NeoRequest<String> {
		NeoRequest::new(
			"getstate",
			vec![root_hash.to_value(), contract_hash.to_value(), key.to_value()], //key.to_base64()],
		)
	}
	fn find_states(
		&self,
		root_hash: H256,
		contract_hash: H160,
		key_prefix: &str,
		start_key: Option<&str>,
		count: Option<u32>,
	) -> NeoRequest<States> {
		let mut params =
			vec![root_hash.to_value(), contract_hash.to_value(), key_prefix.to_value()];
		if let Some(start_key) = start_key {
			params.push(start_key.to_value())
		}
		if let Some(count) = count {
			params.push(count.to_value())
		}

		NeoRequest::new("findstates", params)
	}

	fn get_block_by_index(&self, index: u32, full_tx: bool) -> NeoRequest<NeoBlock> {
		let full_tx = if full_tx { 1 } else { 0 };
		NeoRequest::new("getblock", vec![index.to_value(), full_tx.to_value()])
	}

	fn get_raw_block_by_index(&self, index: u32) -> NeoRequest<String> {
		NeoRequest::new("getblock", vec![index.to_value(), 0.to_value()])
	}

	fn invoke_function_diagnostics(
		&self,
		contract_hash: H160,
		name: String,
		params: Vec<ContractParameter>,
		signers: Vec<Signer>,
	) -> NeoRequest<InvocationResult> {
		let params = vec![
			contract_hash.to_value(),
			name.to_value(),
			serde_json::to_string(&params).unwrap().to_value(),
			serde_json::to_string(&signers).unwrap().to_value(),
			true.to_value(),
		];

		NeoRequest::new("invokefunction", params)
	}

	fn invoke_script_diagnostics(
		&self,
		hex: String,
		signers: Vec<Signer>,
	) -> NeoRequest<InvocationResult> {
		let params = vec![hex.to_value(), signers.to_value(), true.to_value()];

		NeoRequest::new("invokescript", params)
	}

	fn traverse_iterator(
		&self,
		session_id: String,
		iterator_id: String,
		count: u32,
	) -> NeoRequest<Vec<StackItem>> {
		let params = vec![session_id.to_value(), iterator_id.to_value(), count.to_value()];

		NeoRequest::new("traverseiterator", params)
	}

	fn terminate_session(&self, session_id: &str) -> NeoRequest<bool> {
		NeoRequest::new("terminatesession", vec![session_id.to_value()])
	}

	fn invoke_contract_verify(
		&self,
		hash: H160,
		params: Vec<ContractParameter>,
		signers: Vec<Signer>,
	) -> NeoRequest<InvocationResult> {
		NeoRequest::new(
			"invokecontractverify",
			vec![hash.to_value(), params.to_value(), signers.to_value()],
		)
	}

	fn get_raw_mempool(&self) -> NeoRequest<MemPoolDetails> {
		NeoRequest::new("getrawmempool", vec![])
	}

	fn import_private_key(&self, wif: String) -> NeoRequest<NeoAddress> {
		NeoRequest::new("importprivkey", vec![wif.to_value()])
	}

	fn get_block_header_hash(&self, hash: H256) -> NeoRequest<NeoBlock> {
		NeoRequest::new("getblockheader", vec![hash.to_value(), 1.to_value()])
	}

	fn send_to_address_send_token(
		&self,
		send_token: &TransactionSendToken,
	) -> NeoRequest<Transaction> {
		let params = [send_token.to_value()].to_vec();
		NeoRequest::new("sendtoaddress", params)
	}

	fn send_from_send_token(
		&self,
		send_token: &TransactionSendToken,
		from: Address,
	) -> NeoRequest<Transaction> {
		let params = [from.to_value(), vec![send_token.to_value()].into()].to_vec();
		NeoRequest::new("sendmany", params)
	}
}

impl<P: JsonRpcClient> Provider<P> {
	async fn query_resolver<T: Detokenize>(
		&self,
		param: ParamType,
		nns_name: &str,
		selector: Selector,
	) -> Result<T, ProviderError> {
		self.query_resolver_parameters(param, nns_name, selector, None).await
	}

	async fn query_resolver_parameters<T: Detokenize>(
		&self,
		param: ParamType,
		nns_name: &str,
		selector: Selector,
		parameters: Option<&[u8]>,
	) -> Result<T, ProviderError> {
		// Get the NNS address, prioritize the local override variable
		let ens_addr = self.nns.unwrap_or(nns::ENS_ADDRESS);

		// first get the resolver responsible for this name
		// the call will return a Bytes array which we convert to an address
		let data = self.call(&nns::get_resolver(ens_addr, nns_name).into(), None).await?;

		// otherwise, decode_bytes panics
		if data.0.is_empty() {
			return Err(ProviderError::EnsError(nns_name.to_string()))
		}

		let resolver_address: Address = decode_bytes(ParamType::Address, data);
		if resolver_address == Address::zero() {
			return Err(ProviderError::EnsError(nns_name.to_string()))
		}

		if let ParamType::Address = param {
			// Reverse resolver reverts when calling `supportsInterface(bytes4)`
			self.validate_resolver(resolver_address, selector, nns_name).await?;
		}

		// resolve
		let data = self
			.call(&nns::resolve(resolver_address, selector, nns_name, parameters).into(), None)
			.await?;

		Ok(decode_bytes(param, data))
	}

	/// Validates that the resolver supports `selector`.
	async fn validate_resolver(
		&self,
		resolver_address: Address,
		selector: Selector,
		nns_name: &str,
	) -> Result<(), ProviderError> {
		let data = self
			.call(&nns::supports_interface(resolver_address, selector).into(), None)
			.await?;

		if data.is_empty() {
			return Err(ProviderError::EnsError(format!(
				"`{nns_name}` resolver ({resolver_address:?}) is invalid."
			)))
		}

		let supports_selector = abi::decode(&[ParamType::Bool], data.as_ref())
			.map(|token| token[0].clone().into_bool().unwrap_or_default())
			.unwrap_or_default();

		if !supports_selector {
			return Err(ProviderError::EnsError(format!(
				"`{}` resolver ({:?}) does not support selector {}.",
				nns_name,
				resolver_address,
				hex::encode(selector)
			)))
		}

		Ok(())
	}

	#[cfg(test)]
	/// Anvil and Ganache-only function for mining empty blocks
	pub async fn mine(&self, num_blocks: usize) -> Result<(), ProviderError> {
		for _ in 0..num_blocks {
			self.inner
				.request::<_, U256>("evm_mine", None::<()>)
				.await
				.map_err(Into::into)?;
		}
		Ok(())
	}

	/// Sets the NNS Address (default: mainnet)
	#[must_use]
	pub fn nns<T: Into<Address>>(mut self, nns: T) -> Self {
		self.nns = Some(nns.into());
		self
	}

	/// Sets the default polling interval for event filters and pending transactions
	/// (default: 7 seconds)
	pub fn set_interval<T: Into<Duration>>(&mut self, interval: T) -> &mut Self {
		self.interval = Some(interval.into());
		self
	}

	/// Sets the default polling interval for event filters and pending transactions
	/// (default: 7 seconds)
	#[must_use]
	pub fn interval<T: Into<Duration>>(mut self, interval: T) -> Self {
		self.set_interval(interval);
		self
	}

	/// Gets the polling interval which the provider currently uses for event filters
	/// and pending transactions (default: 7 seconds)
	pub fn get_interval(&self) -> Duration {
		self.interval.unwrap_or(DEFAULT_POLL_INTERVAL)
	}
}

#[cfg(all(feature = "ipc", any(unix, windows)))]
impl Provider<crate::Ipc> {
	#[cfg_attr(unix, doc = "Connects to the Unix socket at the provided path.")]
	#[cfg_attr(windows, doc = "Connects to the named pipe at the provided path.\n")]
	#[cfg_attr(
		windows,
		doc = r"Note: the path must be the fully qualified, like: `\\.\pipe\<name>`."
	)]
	pub async fn connect_ipc(path: impl AsRef<std::path::Path>) -> Result<Self, ProviderError> {
		let ipc = crate::Ipc::connect(path).await?;
		Ok(Self::new(ipc))
	}
}

impl Provider<HttpProvider> {
	/// The Url to which requests are made
	pub fn url(&self) -> &Url {
		self.inner.url()
	}

	/// Mutable access to the Url to which requests are made
	pub fn url_mut(&mut self) -> &mut Url {
		self.inner.url_mut()
	}
}

impl<Read, Write> Provider<RwClient<Read, Write>>
where
	Read: JsonRpcClient + 'static,
	<Read as JsonRpcClient>::Error: Sync + Send + 'static,
	Write: JsonRpcClient + 'static,
	<Write as JsonRpcClient>::Error: Sync + Send + 'static,
{
	/// Creates a new [Provider] with a [RwClient]
	pub fn rw(r: Read, w: Write) -> Self {
		Self::new(RwClient::new(r, w))
	}
}

impl<T: JsonRpcClientWrapper> Provider<QuorumProvider<T>> {
	/// Provider that uses a quorum
	pub fn quorum(inner: QuorumProvider<T>) -> Self {
		Self::new(inner)
	}
}

impl Provider<MockProvider> {
	/// Returns a `Provider` instantiated with an internal "mock" transport.
	///
	/// # Example
	///
	/// ```
	/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
	/// use neo_types::U64;
	/// use neo_providers::{Middleware, Provider};
	/// // Instantiate the provider
	/// let (provider, mock) = Provider::mocked();
	/// // Push the mock response
	/// mock.push(U64::from(12))?;
	/// // Make the call
	/// let blk = provider.get_block_number().await.unwrap();
	/// // The response matches
	/// assert_eq!(blk.as_u64(), 12);
	/// // and the request as well!
	/// mock.assert_request("neo_blockNumber", ()).unwrap();
	/// # Ok(())
	/// # }
	/// ```
	pub fn mocked() -> (Self, MockProvider) {
		let mock = MockProvider::new();
		let mock_clone = mock.clone();
		(Self::new(mock), mock_clone)
	}
}

/// infallible conversion of Bytes to Address/String
///
/// # Panics
///
/// If the provided bytes were not an interpretation of an address
fn decode_bytes<T: Detokenize>(param: ParamType, bytes: Bytes) -> T {
	let tokens = abi::decode(&[param], bytes.as_ref())
		.expect("could not abi-decode bytes to address tokens");
	T::from_tokens(tokens).expect("could not parse tokens as address")
}

impl TryFrom<&str> for Provider<HttpProvider> {
	type Error = ParseError;

	fn try_from(src: &str) -> Result<Self, Self::Error> {
		Ok(Provider::new(HttpProvider::new(Url::parse(src)?)))
	}
}

impl TryFrom<String> for Provider<HttpProvider> {
	type Error = ParseError;

	fn try_from(src: String) -> Result<Self, Self::Error> {
		Provider::try_from(src.as_str())
	}
}

impl<'a> TryFrom<&'a String> for Provider<HttpProvider> {
	type Error = ParseError;

	fn try_from(src: &'a String) -> Result<Self, Self::Error> {
		Provider::try_from(src.as_str())
	}
}

#[cfg(not(target_arch = "wasm32"))]
impl Provider<RetryClient<HttpProvider>> {
	/// Create a new [`RetryClient`] by connecting to the provided URL. Errors
	/// if `src` is not a valid URL
	pub fn new_client(src: &str, max_retry: u32, initial_backoff: u64) -> Result<Self, ParseError> {
		Ok(Provider::new(RetryClient::new(
			HttpProvider::new(Url::parse(src)?),
			Box::new(HttpRateLimitRetryPolicy),
			max_retry,
			initial_backoff,
		)))
	}
}

mod sealed {
	use crate::{Http, Provider};
	/// private trait to ensure extension trait is not implement outside of this crate
	pub trait Sealed {}
	impl Sealed for Provider<Http> {}
}

/// Extension trait for `Provider`
///
/// **Note**: this is currently sealed until <https://github.com/gakonst/neo-rs/pull/1267> is finalized
///
/// # Example
///
/// Automatically configure poll interval via `neo_getChainId`
///
/// Note that this will send an RPC to retrieve the network magic.
///
/// ```no_run
///  # use neo_providers::{Http, Provider, ProviderExt};
///  # async fn t() {
/// let http_provider = Provider::<Http>::connect("https://eth.llamarpc.com").await;
/// # }
/// ```
///
/// This is essentially short for
///
/// ```no_run
/// use std::convert::TryFrom;
/// use neo_types::Chain;
/// use neo_providers::{Http, Provider, ProviderExt};
/// let http_provider = Provider::<Http>::try_from("https://eth.llamarpc.com").unwrap().set_chain(Chain::Mainnet);
/// ```
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait ProviderExt: sealed::Sealed {
	/// The error type that can occur when creating a provider
	type Error: Debug;

	/// Creates a new instance connected to the given `url`, exit on error
	async fn connect(url: &str) -> Self
	where
		Self: Sized,
	{
		Self::try_connect(url).await.unwrap()
	}

	/// Try to create a new `Provider`
	async fn try_connect(url: &str) -> Result<Self, Self::Error>
	where
		Self: Sized;

	/// Customize `Provider` settings for chain.
	///
	/// E.g. [`Chain::average_blocktime_hint()`] returns the average block time which can be used to
	/// tune the polling interval.
	///
	/// Returns the customized `Provider`
	fn for_chain(mut self, chain: impl Into<Chain>) -> Self
	where
		Self: Sized,
	{
		self.set_chain(chain);
		self
	}

	/// Customized `Provider` settings for chain
	fn set_chain(&mut self, chain: impl Into<Chain>) -> &mut Self;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl ProviderExt for Provider<HttpProvider> {
	type Error = ParseError;

	async fn try_connect(url: &str) -> Result<Self, Self::Error>
	where
		Self: Sized,
	{
		let mut provider = Provider::try_from(url)?;
		if is_local_endpoint(url) {
			provider.set_interval(DEFAULT_LOCAL_POLL_INTERVAL);
		} else if let Some(chain) =
			provider.get_network_magic().await.ok().and_then(|id| Chain::try_from(id).ok())
		{
			provider.set_chain(chain);
		}

		Ok(provider)
	}

	fn set_chain(&mut self, chain: impl Into<Chain>) -> &mut Self {
		let chain = chain.into();
		if let Some(blocktime) = chain.average_blocktime_hint() {
			// use half of the block time
			self.set_interval(blocktime / 2);
		}
		self
	}
}

/// Returns true if the endpoint is local
///
/// # Example
///
/// ```
/// use neo_providers::is_local_endpoint;
/// assert!(is_local_endpoint("http://localhost:8545"));
/// assert!(is_local_endpoint("http://test.localdev.me"));
/// assert!(is_local_endpoint("http://169.254.0.0:8545"));
/// assert!(is_local_endpoint("http://127.0.0.1:8545"));
/// assert!(!is_local_endpoint("http://206.71.50.230:8545"));
/// assert!(!is_local_endpoint("http://[2001:0db8:85a3:0000:0000:8a2e:0370:7334]"));
/// assert!(is_local_endpoint("http://[::1]"));
/// assert!(!is_local_endpoint("havenofearlucishere"));
/// ```
#[inline]
pub fn is_local_endpoint(endpoint: &str) -> bool {
	if let Ok(url) = Url::parse(endpoint) {
		if let Some(host) = url.host() {
			match host {
				Host::Domain(domain) =>
					return domain.contains("localhost") || domain.contains("localdev.me"),
				Host::Ipv4(ipv4) =>
					return ipv4 == Ipv4Addr::LOCALHOST
						|| ipv4.is_link_local() || ipv4.is_loopback()
						|| ipv4.is_private(),
				Host::Ipv6(ipv6) => return ipv6.is_loopback(),
			}
		}
	}
	false
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Http;
	use futures_util::StreamExt;
	use std::path::PathBuf;

	#[test]
	fn convert_h256_u256_quantity() {
		let hash: H256 = H256::zero();
		let quantity = U256::from_big_endian(hash.as_bytes());
		assert_eq!(format!("{quantity:#x}"), "0x0");
		assert_eq!(utils::serialize(&quantity).to_string(), "\"0x0\"");

		let address: Address = "0x295a70b2de5e3953354a6a8344e616ed314d7251".parse().unwrap();
		let block = BlockNumber::Latest;
		let params =
			[utils::serialize(&address), utils::serialize(&quantity), utils::serialize(&block)];

		let params = serde_json::to_string(&params).unwrap();
		assert_eq!(params, r#"["0x295a70b2de5e3953354a6a8344e616ed314d7251","0x0","latest"]"#);
	}

	// Test vector from: https://docs.neo.io/neo.js/v5-beta/api-providers.html#id2
	#[tokio::test]
	async fn mainnet_resolve_name() {
		let provider = crate::test_provider::MAINNET.provider();

		let addr = provider.resolve_name("registrar.firefly.eth").await.unwrap();
		assert_eq!(addr, "6fC21092DA55B392b045eD78F4732bff3C580e2c".parse().unwrap());

		// registrar not found
		provider.resolve_name("asdfasdffads").await.unwrap_err();

		// name not found
		provider.resolve_name("asdfasdf.registrar.firefly.eth").await.unwrap_err();
	}

	// Test vector from: https://docs.neo.io/neo.js/v5-beta/api-providers.html#id2
	#[tokio::test]
	async fn mainnet_lookup_address() {
		let provider = crate::MAINNET.provider();

		let name = provider
			.lookup_address("6fC21092DA55B392b045eD78F4732bff3C580e2c".parse().unwrap())
			.await
			.unwrap();

		assert_eq!(name, "registrar.firefly.eth");

		provider
			.lookup_address("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".parse().unwrap())
			.await
			.unwrap_err();
	}

	#[tokio::test]
	#[ignore]
	async fn mainnet_resolve_avatar() {
		let provider = crate::MAINNET.provider();

		for (nns_name, res) in &[
            // HTTPS
            ("alisha.eth", "https://ipfs.io/ipfs/QmeQm91kAdPGnUKsE74WvkqYKUeHvc2oHd2FW11V3TrqkQ"),
            // ERC-1155
            ("nick.eth", "https://img.seadn.io/files/3ae7be6c41ad4767bf3ecbc0493b4bfb.png"),
            // HTTPS
            ("parishilton.eth", "https://i.imgur.com/YW3Hzph.jpg"),
            // ERC-721 with IPFS link
            ("ikehaya-nft.eth", "https://ipfs.io/ipfs/QmdKkwCE8uVhgYd7tWBfhtHdQZDnbNukWJ8bvQmR6nZKsk"),
            // ERC-1155 with IPFS link
            ("vitalik.eth", "https://ipfs.io/ipfs/QmSP4nq9fnN9dAiCj42ug9Wa79rqmQerZXZch82VqpiH7U/image.gif"),
            // IPFS
            ("cdixon.eth", "https://ipfs.io/ipfs/QmYA6ZpEARgHvRHZQdFPynMMX8NtdL2JCadvyuyG2oA88u"),
            ("0age.eth", "data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iVVRGLTgiPz48c3ZnIHN0eWxlPSJiYWNrZ3JvdW5kLWNvbG9yOmJsYWNrIiB2aWV3Qm94PSIwIDAgNTAwIDUwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cmVjdCB4PSIxNTUiIHk9IjYwIiB3aWR0aD0iMTkwIiBoZWlnaHQ9IjM5MCIgZmlsbD0iIzY5ZmYzNyIvPjwvc3ZnPg==")
        ] {
        println!("Resolving: {nns_name}");
        assert_eq!(provider.resolve_avatar(nns_name).await.unwrap(), Url::parse(res).unwrap());
    }
	}

	#[tokio::test]
	#[cfg_attr(feature = "celo", ignore)]
	async fn test_new_block_filter() {
		let num_blocks = 3;
		let geth = Anvil::new().block_time(2u64).spawn();
		let provider = Provider::<Http>::try_from(geth.endpoint())
			.unwrap()
			.interval(Duration::from_millis(1000));

		let start_block = provider.get_block_number().await.unwrap();

		let stream = provider.watch_blocks().await.unwrap().stream();

		let hashes: Vec<H256> = stream.take(num_blocks).collect::<Vec<H256>>().await;
		for (i, hash) in hashes.iter().enumerate() {
			let block = provider.get_block(start_block + i as u64 + 1).await.unwrap().unwrap();
			assert_eq!(*hash, block.hash.unwrap());
		}
	}

	#[tokio::test]
	async fn test_is_signer() {
		use std::str::FromStr;

		let anvil = Anvil::new().spawn();
		let provider = Provider::<Http>::try_from(anvil.endpoint())
			.unwrap()
			.with_sender(anvil.addresses()[0]);
		assert!(provider.is_signer().await);

		let provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap();
		assert!(!provider.is_signer().await);

		let sender = Address::from_str("635B4764D1939DfAcD3a8014726159abC277BecC")
			.expect("should be able to parse hex address");
		let provider = Provider::<Http>::try_from(
			"https://ropsten.infura.io/v3/fd8b88b56aa84f6da87b60f5441d6778",
		)
		.unwrap()
		.with_sender(sender);
		assert!(!provider.is_signer().await);
	}

	#[tokio::test]
	async fn test_new_pending_txs_filter() {
		let num_txs = 5;

		let geth = Anvil::new().block_time(2u64).spawn();
		let provider = Provider::<Http>::try_from(geth.endpoint())
			.unwrap()
			.interval(Duration::from_millis(1000));
		let accounts = provider.get_accounts().await.unwrap();

		let stream = provider.watch_pending_transactions().await.unwrap().stream();

		let mut tx_hashes = Vec::new();
		let tx = TransactionRequest::new().from(accounts[0]).to(accounts[0]).value(1e18 as u64);

		for _ in 0..num_txs {
			tx_hashes.push(provider.send_transaction(tx.clone(), None).await.unwrap());
		}

		let hashes: Vec<H256> = stream.take(num_txs).collect::<Vec<H256>>().await;
		assert_eq!(tx_hashes, hashes);
	}

	#[tokio::test]
	async fn receipt_on_unmined_tx() {
		let anvil = Anvil::new().block_time(2u64).spawn();
		let provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap();

		let accounts = provider.get_accounts().await.unwrap();
		let tx = TransactionRequest::pay(accounts[0], parse_ether(1u64).unwrap()).from(accounts[0]);
		let pending_tx = provider.send_transaction(tx, None).await.unwrap();

		assert!(provider.get_transaction_receipt(*pending_tx).await.unwrap().is_none());

		let hash = *pending_tx;
		let receipt = pending_tx.await.unwrap().unwrap();
		assert_eq!(receipt.transaction_hash, hash);
	}

	#[tokio::test]
	async fn parity_block_receipts() {
		let url = match std::env::var("PARITY") {
			Ok(inner) => inner,
			_ => return,
		};
		let provider = Provider::<Http>::try_from(url.as_str()).unwrap();
		let receipts = provider.parity_block_receipts(10657200).await.unwrap();
		assert!(!receipts.is_empty());
	}

	#[tokio::test]
	#[cfg_attr(feature = "celo", ignore)]
	async fn debug_trace_block() {
		let provider = Provider::<Http>::try_from("https://eth.llamarpc.com").unwrap();

		let opts = GethDebugTracingOptions {
			disable_storage: Some(false),
			tracer: Some(GethDebugTracerType::BuiltInTracer(
				GethDebugBuiltInTracerType::PreStateTracer,
			)),
			tracer_config: Some(GethDebugTracerConfig::BuiltInTracer(
				GethDebugBuiltInTracerConfig::PreStateTracer(PreStateConfig {
					diff_mode: Some(true),
				}),
			)),
			..Default::default()
		};

		let latest_block = provider
			.get_block(BlockNumber::Latest)
			.await
			.expect("Failed to fetch latest block.")
			.expect("Latest block is none.");

		// debug_traceBlockByNumber
		let latest_block_num = BlockNumber::Number(latest_block.number.unwrap());
		let traces_by_num = provider
			.debug_trace_block_by_number(Some(latest_block_num), opts.clone())
			.await
			.unwrap();
		for trace in &traces_by_num {
			assert!(matches!(trace, GethTrace::Known(..)));
		}

		// debug_traceBlockByHash
		let latest_block_hash = latest_block.hash.unwrap();
		let traces_by_hash =
			provider.debug_trace_block_by_hash(latest_block_hash, opts).await.unwrap();
		for trace in &traces_by_hash {
			assert!(matches!(trace, GethTrace::Known(..)));
		}

		assert_eq!(traces_by_num, traces_by_hash);
	}

	#[tokio::test]
	#[cfg_attr(feature = "celo", ignore)]
	async fn fee_history() {
		let provider = Provider::<Http>::try_from(
			"https://goerli.infura.io/v3/fd8b88b56aa84f6da87b60f5441d6778",
		)
		.unwrap();

		provider.fee_history(10u64, BlockNumber::Latest, &[10.0, 40.0]).await.unwrap();
	}

	#[tokio::test]
	#[ignore]
	#[cfg(feature = "ws")]
	async fn test_trace_call_many() {
		use neo_types::H160;

		// TODO: Implement ErigonInstance, so it'd be possible to test this.
		let provider = Provider::new(crate::Ws::connect("ws://127.0.0.1:8545").await.unwrap());
		provider
			.trace_call_many(
				vec![
					(
						TransactionRequest::new()
							.from(Address::zero())
							.to("0x0000000000000000000000000000000000000001"
								.parse::<H160>()
								.unwrap())
							.value(U256::from(10000000000000000u128)),
						vec![TraceType::StateDiff],
					),
					(
						TransactionRequest::new()
							.from(
								"0x0000000000000000000000000000000000000001"
									.parse::<H160>()
									.unwrap(),
							)
							.to("0x0000000000000000000000000000000000000002"
								.parse::<H160>()
								.unwrap())
							.value(U256::from(10000000000000000u128)),
						vec![TraceType::StateDiff],
					),
				],
				None,
			)
			.await
			.unwrap();
	}

	#[tokio::test]
	async fn test_fill_transaction_1559() {
		let (mut provider, mock) = Provider::mocked();
		provider.from = Some("0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse().unwrap());

		let gas = U256::from(21000_usize);
		let max_fee = U256::from(25_usize);
		let prio_fee = U256::from(25_usize);
		let access_list: AccessList = vec![Default::default()].into();

		// --- leaves a filled 1559 transaction unchanged, making no requests
		let from: Address = "0x0000000000000000000000000000000000000001".parse().unwrap();
		let to: Address = "0x0000000000000000000000000000000000000002".parse().unwrap();
		let mut tx = Eip1559TransactionRequest::new()
			.from(from)
			.to(to)
			.gas(gas)
			.max_fee_per_gas(max_fee)
			.max_priority_fee_per_gas(prio_fee)
			.access_list(access_list.clone())
			.into();
		provider.fill_transaction(&mut tx, None).await.unwrap();

		assert_eq!(tx.from(), Some(&from));
		assert_eq!(tx.to(), Some(&to.into()));
		assert_eq!(tx.gas(), Some(&gas));
		assert_eq!(tx.gas_price(), Some(max_fee));
		assert_eq!(tx.access_list(), Some(&access_list));

		// --- fills a 1559 transaction, leaving the existing gas limit unchanged,
		// without generating an access-list
		let mut tx = Eip1559TransactionRequest::new()
			.gas(gas)
			.max_fee_per_gas(max_fee)
			.max_priority_fee_per_gas(prio_fee)
			.into();

		provider.fill_transaction(&mut tx, None).await.unwrap();

		assert_eq!(tx.from(), provider.from.as_ref());
		assert!(tx.to().is_none());
		assert_eq!(tx.gas(), Some(&gas));
		assert_eq!(tx.access_list(), Some(&Default::default()));

		// --- fills a 1559 transaction, using estimated gas
		let mut tx = Eip1559TransactionRequest::new()
			.max_fee_per_gas(max_fee)
			.max_priority_fee_per_gas(prio_fee)
			.into();

		mock.push(gas).unwrap();

		provider.fill_transaction(&mut tx, None).await.unwrap();

		assert_eq!(tx.from(), provider.from.as_ref());
		assert!(tx.to().is_none());
		assert_eq!(tx.gas(), Some(&gas));
		assert_eq!(tx.access_list(), Some(&Default::default()));

		// --- propogates estimate_gas() error
		let mut tx = Eip1559TransactionRequest::new()
			.max_fee_per_gas(max_fee)
			.max_priority_fee_per_gas(prio_fee)
			.into();

		// bad mock value causes error response for neo_estimateGas
		mock.push(b'b').unwrap();

		let res = provider.fill_transaction(&mut tx, None).await;

		assert!(matches!(res, Err(ProviderError::JsonRpcClientError(_))));
	}

	#[tokio::test]
	async fn test_fill_transaction_legacy() {
		let (mut provider, mock) = Provider::mocked();
		provider.from = Some("0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse().unwrap());

		let gas = U256::from(21000_usize);
		let gas_price = U256::from(50_usize);

		// --- leaves a filled legacy transaction unchanged, making no requests
		let from: Address = "0x0000000000000000000000000000000000000001".parse().unwrap();
		let to: Address = "0x0000000000000000000000000000000000000002".parse().unwrap();
		let mut tx =
			TransactionRequest::new().from(from).to(to).gas(gas).gas_price(gas_price).into();
		provider.fill_transaction(&mut tx, None).await.unwrap();

		assert_eq!(tx.from(), Some(&from));
		assert_eq!(tx.to(), Some(&to.into()));
		assert_eq!(tx.gas(), Some(&gas));
		assert_eq!(tx.gas_price(), Some(gas_price));
		assert!(tx.access_list().is_none());

		// --- fills an empty legacy transaction
		let mut tx = TransactionRequest::new().into();
		mock.push(gas).unwrap();
		mock.push(gas_price).unwrap();
		provider.fill_transaction(&mut tx, None).await.unwrap();

		assert_eq!(tx.from(), provider.from.as_ref());
		assert!(tx.to().is_none());
		assert_eq!(tx.gas(), Some(&gas));
		assert_eq!(tx.gas_price(), Some(gas_price));
		assert!(tx.access_list().is_none());
	}

	#[tokio::test]
	async fn mainnet_lookup_address_invalid_resolver() {
		let provider = crate::MAINNET.provider();

		let err = provider
			.lookup_address("0x30c9223d9e3d23e0af1073a38e0834b055bf68ed".parse().unwrap())
			.await
			.unwrap_err();

		assert_eq!(
            &err.to_string(),
            "nns name not found: `ox63616e.eth` resolver (0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2) is invalid."
        );
	}

	#[tokio::test]
	async fn gneo_admin_nodeinfo() {
		// we can't use the test provider because infura does not expose admin endpoints
		let network = 1337u64;
		let dir = tempfile::tempdir().unwrap();

		let (geth, provider) =
			spawn_gneo_and_create_provider(network, Some(dir.path().into()), None);

		let info = provider.node_info().await.unwrap();
		drop(geth);

		// make sure it is running eth
		assert!(info.protocols.eth.is_some());

		// check that the network magic is correct
		assert_eq!(info.protocols.eth.unwrap().network, network);

		#[cfg(not(windows))]
		dir.close().unwrap();
	}

	/// Spawn a new `GethInstance` without discovery and create a `Provider` for it.
	///
	/// These will all use the same genesis config.
	fn spawn_gneo_and_create_provider(
		network_magic: u64,
		datadir: Option<PathBuf>,
		genesis: Option<Genesis>,
	) -> (GethInstance, Provider<HttpProvider>) {
		let geth = Geth::new().network_magic(network_magic).disable_discovery();

		let geth = match genesis {
			Some(genesis) => geth.genesis(genesis),
			None => geth,
		};

		let geth = match datadir {
			Some(dir) => geth.data_dir(dir),
			None => geth,
		}
		.spawn();

		let provider = Provider::try_from(geth.endpoint()).unwrap();
		(geth, provider)
	}

	/// Spawn a set of [`GethInstance`]s with the list of given data directories and [`Provider`]s
	/// for those [`GethInstance`]s without discovery, setting sequential ports for their p2p, rpc,
	/// and authrpc ports.
	fn spawn_gneo_instances<const N: usize>(
		datadirs: [PathBuf; N],
		network_magic: u64,
		genesis: Option<Genesis>,
	) -> [(GethInstance, Provider<HttpProvider>); N] {
		datadirs
			.map(|dir| spawn_gneo_and_create_provider(network_magic, Some(dir), genesis.clone()))
	}

	#[tokio::test]
	#[cfg_attr(windows, ignore = "cannot spawn multiple geth instances")]
	async fn add_second_gneo_peer() {
		// init each geth directory
		let dir1 = tempfile::tempdir().unwrap();
		let dir2 = tempfile::tempdir().unwrap();

		// use the default genesis
		let genesis = utils::Genesis::default();

		// spawn the geths
		let [(mut first_geth, first_peer), (second_geth, second_peer)] =
			spawn_gneo_instances([dir1.path().into(), dir2.path().into()], 1337, Some(genesis));

		// get nodeinfo for each geth instance
		let first_info = first_peer.node_info().await.unwrap();
		let second_info = second_peer.node_info().await.unwrap();
		let first_port = first_info.ports.listener;

		// replace the ip in the enode by putting
		let first_prefix = first_info.enode.split('@').collect::<Vec<&str>>();

		// create enodes for each geth instance using each id and port
		let first_enode = format!("{}@localhost:{}", first_prefix.first().unwrap(), first_port);

		// add the first geth as a peer for the second
		let res = second_peer.add_peer(first_enode).await.unwrap();
		assert!(res);

		// wait on the listening peer for an incoming connection
		first_geth.wait_to_add_peer(second_info.id).unwrap();

		// check that second_geth exists in the first_geth peer list
		let peers = first_peer.peers().await.unwrap();

		drop(first_geth);
		drop(second_geth);

		// check that the second peer is in the list (it uses an enr so the enr should be Some)
		assert_eq!(peers.len(), 1);

		let peer = peers.get(0).unwrap();
		assert_eq!(H256::from_str(&peer.id).unwrap(), second_info.id);

		// remove directories
		#[cfg(not(windows))]
		{
			dir1.close().unwrap();
			dir2.close().unwrap();
		}
	}
}
