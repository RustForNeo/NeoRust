// fungible_token

use crate::contract::token::Token;
use crate::{
	contract::nns_name::NNSName,
	transaction::transaction_builder::TransactionBuilder,
	types::Bytes,
	wallet::{account::Account, wallet::Wallet},
};
use primitive_types::H160;

pub struct FungibleToken {
	script_hash: H160,
	total_supply: Option<u64>,
	decimals: Option<u8>,
	symbol: Option<String>,
}

impl<T> FungibleToken {
	const BALANCE_OF: &'static str = "balanceOf";
	const TRANSFER: &'static str = "transfer";

	async fn get_balance(&self, account: &Account) -> u64 {
		self.get_balance(&account.script_hash()).await
	}

	async fn get_balance_hash256(&self, script_hash: &H160) -> u64 {
		self.call_function_u64(FungibleToken::BALANCE_OF, vec![script_hash]).await
	}

	async fn transfer(&self, from: &Account, to: &H160, amount: u64) -> TransactionBuilder<T> {
		let script = self.build_transfer_script(from.script_hash(), to, amount).await;
		TransactionBuilder::new().script(script).signer(from)
	}

	async fn build_transfer_script(&self, from: &H160, to: &H160, amount: u64) -> Bytes {
		self.build_invoke_function_script(FungibleToken::TRANSFER, vec![from, to, amount])
			.await
	}

	// other methods
	async fn get_wallet_balance(&self, wallet: &Wallet) -> u64 {
		let mut total = 0;

		for account in wallet.accounts() {
			let balance = self.get_balance(&account).await;
			total += balance;
		}

		total
	}
	async fn transfer_to_nns(
		&self,
		from: &Account,
		to: &NNSName,
		amount: u64,
	) -> TransactionBuilder<T> {
		let resolver = NnsResolver::default();
		let to_script_hash = resolver.resolve(to).await;

		self.transfer(from, &to_script_hash, amount).await
	}

	async fn transfer_to_nns_hash160(
		&self,
		from: &Account,
		to: &H160,
		amount: u64,
	) -> TransactionBuilder<T> {
		self.transfer(from, to, amount).await
	}

	async fn mint(&self, to: &Account, amount: u64) -> TransactionBuilder<T> {
		let script = self.build_mint_script(to, amount).await;

		TransactionBuilder::new().script(script)
	}
	async fn build_mint_script(&self, to: &Account, amount: u64) -> Bytes {
		self.build_invoke_function_script(FungibleToken::MINT, vec![to.script_hash(), amount])
			.await
	}
}

impl Token for FungibleToken {
	fn script_hash(&self) -> H160 {
		self.script_hash
	}

	fn set_script_hash(&mut self, script_hash: H160) {
		self.script_hash = script_hash;
	}

	fn total_supply(&self) -> Option<u64> {
		self.total_supply
	}

	fn set_total_supply(&mut self, total_supply: u64) {
		self.total_supply = Some(total_supply);
	}

	fn decimals(&self) -> Option<u8> {
		self.decimals
	}

	fn set_decimals(&mut self, decimals: u8) {
		self.decimals = Some(decimals);
	}

	fn symbol(&self) -> Option<String> {
		self.symbol.clone()
	}

	fn set_symbol(&mut self, symbol: String) {
		self.symbol = Some(symbol);
	}
}
