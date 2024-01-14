use crate::{get_wallet, spawn_anvil, spawn_anvil_ws};
use neo_middleware::{signer::SignerMiddleware, MiddlewareBuilder};
use neo_providers::{
	core::{transaction::transaction::Transaction, wallet::WalletTrait},
	JsonRpcClient, Middleware,
};
use neo_signers::{LocalWallet, Signer};
use neo_types::*;

#[tokio::test]
async fn send_eth() {
	let (provider, anvil) = spawn_anvil();
	let wallet = get_wallet(&anvil, 0);
	let address = wallet.address();
	let provider = provider.with_signer(wallet);

	let to = anvil.addresses()[1];

	// craft the transaction
	let tx = Transaction::new().to(to).value(10000);

	let balance_before = provider.get_balance(address, None).await.unwrap();

	// send it!
	provider.send_transaction(tx).await.unwrap().await.unwrap().unwrap();

	let balance_after = provider.get_balance(address, None).await.unwrap();

	assert!(balance_before > balance_after);
}

#[tokio::test]
async fn typed_txs() {
	let (provider, anvil) = spawn_anvil();
	let wallet = get_wallet(&anvil, 0);
	let address = wallet.address();
	let provider = provider.with_signer(wallet);

	let nonce = 0;
	let gas_price = provider.get_gas_price().await.unwrap() * 125 / 100;

	let tx = Transaction::new().from(address).to(address).nonce(nonce).gas_price(gas_price);
	let tx1 = provider.send_transaction(tx.clone()).await.unwrap();

	let tx = tx.from(address).to(address).nonce(nonce + 1).with_access_list(vec![]);
	let tx2 = provider.send_transaction(tx).await.unwrap();

	futures_util::join!(check_tx(tx1, 0), check_tx(tx2, 1));
}

#[tokio::test]
async fn send_transaction_handles_tx_from_field() {
	// launch anvil
	let (provider, anvil) = spawn_anvil_ws().await;

	// grab 2 wallets
	let signer: LocalWallet = anvil.keys()[0].clone().into();
	let other: LocalWallet = anvil.keys()[1].clone().into();

	// connect to the network
	let provider = SignerMiddleware::new_with_provider_chain(provider, signer.clone())
		.await
		.unwrap();

	// sending a Transaction with a from field of None should result
	// in a transaction from the signer address
	let request_from_none = Transaction::new();
	let receipt = provider
		.send_transaction(request_from_none)
		.await
		.unwrap()
		.await
		.unwrap()
		.unwrap();
	let sent_tx = provider.get_transaction(receipt.transaction_hash).await.unwrap().unwrap();

	assert_eq!(sent_tx.from, signer.address());

	// sending a Transaction with the signer as the from address should
	// result in a transaction from the signer address
	let request_from_signer = Transaction::new().from(signer.address());
	let receipt = provider
		.send_transaction(request_from_signer)
		.await
		.unwrap()
		.await
		.unwrap()
		.unwrap();
	let sent_tx = provider.get_transaction(receipt.hash).await.unwrap().unwrap();

	assert_eq!(sent_tx.sender, signer.default_account());

	// sending a Transaction with a from address that is not the signer
	// should result in a transaction from the specified address
	let request_from_other = Transaction::new().from(other.address());
	let receipt = provider
		.send_transaction(request_from_other)
		.await
		.unwrap()
		.await
		.unwrap()
		.unwrap();
	let sent_tx = provider.get_transaction(receipt.transaction_hash).await.unwrap().unwrap();

	assert_eq!(sent_tx.from, other.address());
}

async fn check_tx<P: JsonRpcClient + Clone>(
	pending_tx: neo_providers::PendingTransaction<'_, P>,
	expected: u64,
) {
	let provider = pending_tx.provider();
	let receipt = pending_tx.await.unwrap().unwrap();
	let tx = provider.get_transaction(receipt.transaction_hash).await.unwrap().unwrap();

	let expected = U64::from(expected);
	for ty in [receipt.transaction_type, tx.transaction_type] {
		// legacy can be either None or Some(0)
		if expected.is_zero() {
			assert!(ty.is_none() || ty == Some(0.into()));
		} else {
			assert_eq!(ty, Some(expected));
		}
	}
}
