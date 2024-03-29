use neo_middleware::{builder::MiddlewareBuilder, signer::SignerMiddleware};
use neo_providers::{Middleware, Provider};
use neo_signers::{LocalWallet, Signer};

#[tokio::test]
async fn build_raw_middleware_stack() {
	let (provider, mock) = Provider::mocked();

	let signer = LocalWallet::new(&mut thread_rng());
	let address = signer.address();
	let escalator = GeometricGasPrice::new(1.125, 60u64, None::<u64>);

	let provider = provider
		.wrap_into(|p| GasEscalatorMiddleware::new(p, escalator, Frequency::PerBlock))
		.wrap_into(|p| GasOracleMiddleware::new(p, GasNow::new()))
		.wrap_into(|p| SignerMiddleware::new(p, signer))
		.wrap_into(|p| NonceManagerMiddleware::new(p, address));

	// push a response
	mock.push(U64::from(12u64)).unwrap();
	let block: U64 = provider.get_block_number().await.unwrap();
	assert_eq!(block.as_u64(), 12);

	provider.get_block_number().await.unwrap_err();

	// 2 calls were made
	mock.assert_request("neo_blockNumber", ()).unwrap();
	mock.assert_request("neo_blockNumber", ()).unwrap();
	mock.assert_request("neo_blockNumber", ()).unwrap_err();
}

#[tokio::test]
async fn build_declarative_middleware_stack() {
	let (provider, mock) = Provider::mocked();

	let signer = LocalWallet::new(&mut thread_rng());
	let address = signer.address();
	let escalator = GeometricGasPrice::new(1.125, 60u64, None::<u64>);
	let gas_oracle = GasNow::new();

	let provider = provider
		.wrap_into(|p| GasEscalatorMiddleware::new(p, escalator, Frequency::PerBlock))
		.gas_oracle(gas_oracle)
		.with_signer(signer);

	// push a response
	mock.push(U64::from(12u64)).unwrap();
	let block: U64 = provider.get_block_number().await.unwrap();
	assert_eq!(block.as_u64(), 12);

	provider.get_block_number().await.unwrap_err();

	// 2 calls were made
	mock.assert_request("neo_blockNumber", ()).unwrap();
	mock.assert_request("neo_blockNumber", ()).unwrap();
	mock.assert_request("neo_blockNumber", ()).unwrap_err();
}
