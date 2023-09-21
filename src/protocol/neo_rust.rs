use crate::{
	neo_error::NeoError,
	protocol::{
		core::{
			neo_trait::NeoTrait,
			request::NeoRequest,
			responses::{
				contract_state::ContractState,
				invocation_result::InvocationResult,
				native_contract_state::NativeContractState,
				neo_address::NeoAddress,
				neo_application_log::NeoApplicationLog,
				neo_block::NeoBlock,
				neo_find_states::States,
				neo_get_mem_pool::MemPoolDetails,
				neo_get_nep11balances::{Nep11Balance, Nep11Balances},
				neo_get_nep11transfers::Nep11Transfers,
				neo_get_nep17balances::Nep17Balances,
				neo_get_nep17transfers::Nep17Transfers,
				neo_get_next_block_validators::Validator,
				neo_get_peers::Peers,
				neo_get_state_height::StateHeight,
				neo_get_state_root::StateRoot,
				neo_get_unclaimed_gas::GetUnclaimedGas,
				neo_get_version::NeoVersion,
				neo_get_wallet_balance::Balance,
				neo_list_plugins::Plugin,
				neo_network_fee::NeoNetworkFee,
				neo_send_raw_transaction::RawTransaction,
				neo_validate_address::ValidateAddress,
				transaction::Transaction,
				transaction_send_token::TransactionSendToken,
				transaction_signer::TransactionSigner,
			},
			stack_item::StackItem,
		},
		http_service::HttpService,
		neo_config::NeoConfig,
		neo_service::NeoService,
		rx::json_rpc2::JsonRpc2,
	},
	transaction::signer::Signer,
	types::{
		contract_parameter::ContractParameter, Address, Bytes, ExternBase64, H160Externsion,
		H256Def, ValueExtension,
	},
};
use async_trait::async_trait;
use bitvec::ptr::Mut;
use lazy_static::lazy_static;
use primitive_types::{H160, H256};
use reqwest::Url;
use serde_json::Value;
use std::{
	collections::HashMap,
	str::FromStr,
	sync::{Arc, Mutex, MutexGuard},
};

lazy_static! {
	pub static ref NEO_HTTP_INSTANCE: Arc<Mutex<NeoRust<HttpService>>> =
		Arc::new(Mutex::new(NeoRust::new_http_service()));
}

#[derive(Debug, Clone, Default)]
pub struct NeoRust<T>
where
	T: NeoService,
{
	config: Arc<Mutex<NeoConfig>>,
	neo_service: Arc<Mutex<T>>,
}

impl<T: NeoService> NeoRust<T>
where
	T: NeoService,
{
	pub fn new_http_service() -> Self {
		Self {
			config: Arc::new(Mutex::new(NeoConfig::default())),
			neo_service: Arc::new(Mutex::new(HttpService::new(Url::from_str("").unwrap(), false))),
		}
	}

	pub fn instance() -> MutexGuard<'static, NeoRust<T>> {
		NEO_HTTP_INSTANCE.lock().unwrap()
	}

	pub fn config(&self) -> &NeoConfig {
		&self.config.lock().unwrap()
	}

	pub fn nns_resolver(&self) -> H160 {
		H160::from(self.config().nns_resolver.clone())
	}

	pub fn block_interval(&self) -> u32 {
		self.config().block_interval
	}

	pub fn neo_rx(&self) -> &JsonRpc2 {
		&self.config().scheduledExecutorService
	}
	pub fn polling_interval(&self) -> u32 {
		self.config().polling_interval
	}

	pub fn max_valid_until_block_increment(&self) -> u32 {
		self.config().max_valid_until_block_increment
	}

	pub(crate) fn get_neo_service(&self) -> &T {
		&self.neo_service.lock().unwrap()
	}

	pub fn get_neo_service_mut(&mut self) -> &mut T {
		&mut self.neo_service.lock().as_mut().unwrap()
	}

	pub fn dump_private_key(&self, script_hash: H160) -> NeoRequest<String> {
		NeoRequest::new("dumpprivkey", vec![Value::String(script_hash.to_address())])
	}

	pub async fn get_network_magic_number(&mut self) -> Result<u32, NeoError> {
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
			self.config
				.get_mut()
				.set_network_magic(magic)
				.expect("Unable to set Network Magic Number");
		}
		Ok(self.config().network_magic.unwrap())
	}

	pub async fn get_network_magic_number_bytes(&mut self) -> Result<Bytes, NeoError> {
		let magic_int = self.get_network_magic_number().await.unwrap() & 0xFFFF_FFFF;
		Ok(magic_int.to_be_bytes().to_vec())
	}
}

#[async_trait]
impl<T: NeoService> NeoTrait for NeoRust<T> {
	// Blockchain methods
	fn get_best_block_hash(&self) -> NeoRequest<H256Def> {
		NeoRequest::new("getbestblockhash", vec![])
	}

	fn get_block_hash(&self, block_index: u32) -> NeoRequest<H256Def> {
		NeoRequest::new("getblockhash", [block_index.to_value()].to_vec())
	}

	fn get_block(&self, block_hash: H256, full_tx: bool) -> NeoRequest<NeoBlock> {
		if full_tx {
			NeoRequest::new("getblock", [block_hash.to_value(), 1].to_vec())
		} else {
			self.get_block_header_hash(block_hash)
		}
	}

	// More methods...

	fn get_raw_block(&self, block_hash: H256) -> NeoRequest<String> {
		NeoRequest::new("getblock", vec![block_hash.to_value(), 0])
	}

	// Node methods

	fn get_block_header_count(&self) -> NeoRequest<u32> {
		NeoRequest::new("getblockheadercount", vec![])
	}

	fn get_block_count(&self) -> NeoRequest<u32> {
		NeoRequest::new("getblockcount", vec![])
	}

	fn get_block_header(&self, block_hash: H256) -> NeoRequest<NeoBlock> {
		NeoRequest::new("getblockheader", vec![block_hash.to_value(), 1])
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
		NeoRequest::new("getrawtransaction", vec![hash.to_value(), 1])
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
		let signers = signers.into_iter().map(TransactionSigner::from).collect();
		NeoRequest::new("invokefunction", vec![contract_hash.to_value(), method, params, signers])
	}

	fn invoke_script(&self, hex: String, signers: Vec<Signer>) -> NeoRequest<InvocationResult> {
		let signers: Vec<TransactionSigner> =
			signers.into_iter().map(|signer| TransactionSigner::from(*signer)).collect();

		NeoRequest::new("invokescript", vec![hex.to_value(), signers])
	}

	// More smart contract methods

	fn get_unclaimed_gas(&self, hash: H160) -> NeoRequest<GetUnclaimedGas> {
		NeoRequest::new("getunclaimedgas", vec![hash.to_value()])
	}

	fn list_plugins(&self) -> NeoRequest<Vec<Plugin>> {
		NeoRequest::new("listplugins", vec![]);
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

	fn calculate_network_fee(&self, hex: String) -> NeoRequest<NeoNetworkFee> {
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
		NeoRequest::new("getblock", vec![index.to_value(), 0])
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
		let signers = signers.into_iter().map(TransactionSigner::from).collect();

		NeoRequest::new("invokecontractverify", vec![hash.to_value(), params, signers])
	}

	fn get_raw_mempool(&self) -> NeoRequest<MemPoolDetails> {
		NeoRequest::new("getrawmempool", vec![])
	}

	fn import_private_key(&self, wif: String) -> NeoRequest<NeoAddress> {
		NeoRequest::new("importprivkey", vec![wif.to_value()])
	}

	fn get_block_header_hash(&self, hash: H256) -> NeoRequest<NeoBlock> {
		NeoRequest::new("getblockheader", vec![hash.to_value(), 1])
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
		let params = [from.to_value(), vec![send_token]].to_vec();
		NeoRequest::new("sendmany", params)
	}
}
