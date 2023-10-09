use neo_middleware::{
	gas_escalator::{Frequency, GasEscalatorMiddleware, GeometricGasPrice},
	MiddlewareBuilder,
};
use neo_providers::{Http, Middleware, Provider};
use neo_signers::{LocalWallet, Signer};
use neo_types::*;
use utils::Anvil;

#[tokio::test]
#[ignore]
async fn gas_escalator() {
	let anvil = Anvil::new().block_time(2u64).spawn();
	let network_magic = anvil.network_magic();
	let provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap();

	// wrap with signer
	let wallet: LocalWallet = anvil.keys().first().unwrap().clone().into();
	let wallet = wallet.with_network_magic(network_magic);
	let address = wallet.address();
	let provider = provider.with_signer(wallet);

	// wrap with escalator
	let escalator = GeometricGasPrice::new(5.0, 10u64, Some(2_000_000_000_000u64));
	let provider = GasEscalatorMiddleware::new(provider, escalator, Frequency::Duration(300));

	let nonce = provider.get_transaction_count(address, None).await.unwrap();
	// 1 gwei default base fee
	let gas_price = U256::from(1_000_000_000_u64);
	let tx = TransactionRequest::pay(Address::zero(), 1u64)
		.gas_price(gas_price)
		.nonce(nonce)
		.network_magic(network_magic);

	eprintln!("sending");
	let pending = provider.send_transaction(tx, None).await.expect("could not send");
	eprintln!("waiting");
	let receipt = pending.await.expect("reverted").expect("dropped");
	assert_eq!(receipt.from, address);
	assert_eq!(receipt.to, Some(Address::zero()));
	assert!(receipt.effective_gas_price.unwrap() > gas_price * 2, "{receipt:?}");
}
