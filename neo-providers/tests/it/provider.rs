mod neo_tests {
	use crate::spawn_anvil;
	use neo_providers::{
		core::transaction::transaction::Transaction, Middleware, StreamExt, GOERLI,
	};
	use neo_types::{block::BlockId, Address, BlockId, BlockNumber, Transaction, H256};

	#[tokio::test]
	async fn non_existing_data_works() {
		let provider = GOERLI.provider();

		assert!(provider.get_transaction(H256::zero()).await.unwrap().is_none());
		assert!(provider.get_transaction(H256::zero()).await.unwrap().is_none());
		assert!(provider.get_block(BlockId::Hash(H256::zero())).await.unwrap().is_none());
		assert!(provider
			.get_block_with_txs(BlockId::Hash(H256::zero()))
			.await
			.unwrap()
			.is_none());
	}

	#[tokio::test]
	async fn client_version() {
		let provider = GOERLI.provider();

		// e.g., Geth/v1.10.6-omnibus-1af33248/linux-amd64/go1.16.6
		assert!(provider
			.client_version()
			.await
			.expect("Could not make web3_clientVersion call to provider")
			.starts_with("Geth/v"));
	}

	// Without TLS this would error with "TLS Support not compiled in"
	#[tokio::test]
	#[cfg(all(feature = "ws", any(feature = "openssl", feature = "rustls")))]
	async fn ssl_websocket() {
		let provider = GOERLI.ws().await;
		assert_ne!(provider.get_block_number().await.unwrap(), 0.into());
	}

	#[tokio::test]
	async fn eip1559_fee_estimation() {
		let provider = neo_providers::MAINNET.provider();

		let (_max_fee_per_gas, _max_priority_fee_per_gas) =
			provider.estimate_eip1559_fees(None).await.unwrap();
	}

	#[tokio::test]
	async fn watch_blocks_http() {
		let (provider, _anvil) = spawn_anvil();
		generic_watch_blocks_test(provider).await;
	}

	#[tokio::test]
	#[cfg(feature = "ws")]
	async fn watch_blocks_ws() {
		let (provider, _anvil) = crate::spawn_anvil_ws().await;
		generic_watch_blocks_test(provider).await;
	}

	#[tokio::test]
	#[cfg(feature = "ipc")]
	async fn watch_blocks_ipc() {
		let (provider, _anvil, _ipc) = crate::spawn_anvil_ipc().await;
		generic_watch_blocks_test(provider).await;
	}

	async fn generic_watch_blocks_test<M: Middleware>(provider: M) {
		let stream = provider.watch_blocks().await.unwrap().stream();
		let hashes = stream.take(3).collect::<Vec<H256>>().await;
		let block = provider.get_block(BlockNumber::Latest).await.unwrap().unwrap();
		assert_eq!(block.hash.unwrap(), *hashes.last().unwrap());
	}

	#[tokio::test]
	#[cfg(feature = "ws")]
	async fn subscribe_blocks_ws() {
		let (provider, _anvil) = crate::spawn_anvil_ws().await;
		generic_subscribe_blocks_test(provider).await;
	}

	#[tokio::test]
	#[cfg(feature = "ipc")]
	async fn subscribe_blocks_ipc() {
		let (provider, _anvil, _ipc) = crate::spawn_anvil_ipc().await;
		generic_subscribe_blocks_test(provider).await;
	}

	#[cfg(any(feature = "ws", feature = "ipc"))]
	async fn generic_subscribe_blocks_test<M>(provider: M)
	where
		M: Middleware,
		M::Provider: neo_providers::PubsubClient,
	{
		let stream = provider.subscribe_blocks().await.unwrap();
		let blocks = stream.take(3).collect::<Vec<_>>().await;
		let block = provider.get_block(BlockNumber::Latest).await.unwrap().unwrap();
		assert_eq!(&block, blocks.last().unwrap());
	}

	#[tokio::test]
	async fn send_tx_http() {
		let (provider, anvil) = spawn_anvil();
		generic_send_tx_test(provider, anvil.addresses()[0]).await;
	}

	#[tokio::test]
	#[cfg(feature = "ws")]
	async fn send_tx_ws() {
		let (provider, anvil) = crate::spawn_anvil_ws().await;
		generic_send_tx_test(provider, anvil.addresses()[0]).await;
	}

	#[tokio::test]
	#[cfg(feature = "ipc")]
	async fn send_tx_ipc() {
		let (provider, anvil, _ipc) = crate::spawn_anvil_ipc().await;
		generic_send_tx_test(provider, anvil.addresses()[0]).await;
	}

	async fn generic_send_tx_test<M: Middleware>(provider: M, who: Address) {
		let tx = Transaction::new().to(who).from(who);
		let pending_tx = provider.send_transaction(tx).await.unwrap();
		let tx_hash = *pending_tx;
		let receipt = pending_tx.confirmations(3).await.unwrap().unwrap();
		assert_eq!(receipt.transaction_hash, tx_hash);
	}
}
