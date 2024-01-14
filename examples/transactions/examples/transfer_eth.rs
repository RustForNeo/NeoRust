use eyre::Result;
use neo::{
	core::{types::Transaction, utils::Anvil},
	providers::{Http, Middleware, Provider},
};
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> Result<()> {
	let anvil = Anvil::new().spawn();

	// connect to the network
	let provider = Provider::<Http>::try_from(anvil.endpoint())?;
	let accounts = provider.get_accounts().await?;
	let from = accounts[0];
	let to = accounts[1];

	// craft the tx
	let tx = Transaction::new().to(to).value(1000).from(from); // specify the `from` field so that the client knows which account to use

	let balance_before = provider.get_balance(from, None).await?;
	let nonce1 = 0;

	// broadcast it via the neo_sendTransaction API
	let tx = provider.send_transaction(tx).await?.await?;

	println!("{}", serde_json::to_string(&tx)?);

	let nonce2 = 0;

	let balance_after = provider.get_balance(from, None).await?;
	assert!(balance_after < balance_before);

	println!("Balance before {balance_before}");
	println!("Balance after {balance_after}");

	Ok(())
}
