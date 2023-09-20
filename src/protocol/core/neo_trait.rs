use crate::{
	protocol::core::{
		request::NeoRequest,
		response::NeoResponse,
		responses::{
			contract_state::ContractState,
			invocation_result::InvocationResult,
			native_contract_state::NativeContractState,
			neo_address::NeoAddress,
			neo_application_log::NeoApplicationLog,
			neo_block::NeoBlock,
			neo_find_states::{NeoFindStates, States},
			neo_get_mem_pool::{MemPoolDetails, NeoGetMemPool},
			neo_get_nep11balances::Nep11Balances,
			neo_get_nep11transfers::Nep11Transfers,
			neo_get_nep17balances::Nep17Balances,
			neo_get_nep17transfers::Nep17Transfers,
			neo_get_next_block_validators::{NeoGetNextBlockValidators, Validator},
			neo_get_peers::{NeoGetPeers, Peers},
			neo_get_state_height::{NeoGetStateHeight, StateHeight},
			neo_get_state_root::{NeoGetStateRoot, StateRoot},
			neo_get_unclaimed_gas::{GetUnclaimedGas, NeoGetUnclaimedGas},
			neo_get_version::{NeoGetVersion, NeoVersion},
			neo_get_wallet_balance::{Balance, NeoGetWalletBalance},
			neo_list_plugins::Plugin,
			neo_network_fee::NeoNetworkFee,
			neo_response_aliases::{
				NeoBlockCount, NeoBlockHash, NeoBlockHeaderCount, NeoCalculateNetworkFee,
				NeoCloseWallet, NeoConnectionCount, NeoDumpPrivKey, NeoGetApplicationLog,
				NeoGetCommittee, NeoGetContractState, NeoGetNativeContracts, NeoGetProof,
				NeoGetRawBlock, NeoGetRawMemPool, NeoGetRawTransaction, NeoGetState, NeoGetStorage,
				NeoGetTransaction, NeoGetTransactionHeight, NeoGetWalletUnclaimedGas,
				NeoInvokeContractVerify, NeoInvokeScript, NeoOpenWallet, NeoSendFrom,
				NeoTerminateSession, NeoTraverseIterator, NeoVerifyProof,
			},
			neo_send_raw_transaction::{NeoSendRawTransaction, RawTransaction},
			neo_validate_address::ValidateAddress,
			transaction::Transaction,
			transaction_send_token::TransactionSendToken,
		},
		stack_item::StackItem,
	},
	transaction::signer::Signer,
	types::contract_parameter::ContractParameter,
};
use async_trait::async_trait;
use primitive_types::{H160, H256};
use std::collections::HashMap;

#[async_trait]
pub trait NeoTrait {
	// Blockchain Methods

	async fn get_best_block_hash(&self) -> NeoRequest<H256>;

	async fn get_block_hash(&self, block_index: i32) -> NeoRequest<H256>;

	async fn get_block(&self, block_hash: H256, return_full_tx: bool) -> NeoRequest<NeoBlock>;

	async fn get_raw_block(&self, block_hash: H256) -> NeoRequest<String>;

	async fn get_block_header_count(&self) -> NeoRequest<i32>;

	async fn get_block_count(&self) -> NeoRequest<i32>;

	async fn get_block_header(&self, block_hash: H256) -> NeoRequest<NeoBlock>;

	async fn get_block_header_by_index(&self, index: i32) -> NeoRequest<NeoBlock>;

	async fn get_raw_block_header(&self, block_hash: H256) -> NeoRequest<String>;

	async fn get_raw_block_header_by_index(&self, index: i32) -> NeoRequest<String>;

	async fn get_native_contracts(&self) -> NeoRequest<Vec<NativeContractState>>;

	async fn get_contract_state(&self, contract_hash: H160) -> NeoRequest<ContractState>;

	async fn get_native_contract_state(&self, name: &str) -> NeoRequest<ContractState>;

	async fn get_mem_pool(&self) -> NeoRequest<MemPoolDetails>;

	async fn get_raw_mem_pool(&self) -> NeoRequest<Vec<H256>>;

	async fn get_transaction(&self, tx_hash: H256) -> NeoRequest<Transaction>;

	async fn get_raw_transaction(&self, tx_hash: H256) -> NeoRequest<String>;

	async fn get_storage(&self, contract_hash: H160, key: &str) -> NeoRequest<String>;

	async fn get_transaction_height(&self, tx_hash: H256) -> NeoRequest<i32>;

	async fn get_next_block_validators(&self) -> NeoRequest<Vec<Validator>>;

	async fn get_committee(&self) -> NeoRequest<Vec<String>>;

	// Node Methods

	async fn get_connection_count(&self) -> NeoRequest<i32>;

	async fn get_peers(&self) -> NeoRequest<Peers>;

	async fn get_version(&self) -> NeoRequest<NeoVersion>;

	async fn send_raw_transaction(&self, hex: String) -> NeoRequest<RawTransaction>;

	async fn submit_block(&self, hex: String) -> NeoRequest<bool>;

	// Smart Contract Methods

	async fn invoke_function(
		&self,
		contract_hash: H160,
		name: String,
		params: Vec<ContractParameter>,
		signers: Vec<Box<dyn Signer>>,
	) -> NeoRequest<InvocationResult>;

	async fn invoke_script(
		&self,
		hex: String,
		signers: Vec<Box<dyn Signer>>,
	) -> NeoRequest<InvocationResult>;

	async fn get_unclaimed_gas(&self, script_hash: H160) -> NeoRequest<GetUnclaimedGas>;

	// Utility Methods

	async fn list_plugins(&self) -> NeoRequest<Vec<Plugin>>;

	async fn validate_address(&self, address: &str) -> NeoRequest<ValidateAddress>;

	// Wallet Methods

	async fn close_wallet(&self) -> NeoRequest<bool>;

	async fn dump_priv_key(&self, script_hash: H160) -> NeoRequest<String>;

	async fn get_wallet_balance(&self, token_hash: H160) -> NeoRequest<Balance>;

	async fn get_new_address(&self) -> NeoRequest<String>;

	async fn get_wallet_unclaimed_gas(&self) -> NeoRequest<String>;

	async fn import_priv_key(&self, wif: String) -> NeoRequest<NeoAddress>;

	async fn calculate_network_fee(&self, hex: String) -> NeoRequest<NeoNetworkFee>;

	async fn list_address(&self) -> NeoRequest<Vec<NeoAddress>>;

	async fn open_wallet(&self, path: String, password: String) -> NeoRequest<bool>;

	async fn send_from(
		&self,
		token_hash: H160,
		from: H160,
		to: H160,
		amount: i32,
	) -> NeoRequest<Transaction>;

	async fn send_many(
		&self,
		from: Option<H160>,
		send_tokens: Vec<TransactionSendToken>,
	) -> NeoRequest<Transaction>;

	async fn send_to_address(
		&self,
		token_hash: H160,
		to: H160,
		amount: i32,
	) -> NeoRequest<Transaction>;

	// Application Logs

	async fn get_application_log(&self, tx_hash: H256) -> NeoRequest<NeoApplicationLog>;

	// TokenTracker Methods

	async fn get_nep17_balances(&self, script_hash: H160) -> NeoRequest<Nep17Balances>;

	async fn get_nep17_transfers(&self, script_hash: H160) -> NeoRequest<Nep17Transfers>;

	async fn get_nep17_transfers_from(
		&self,
		script_hash: H160,
		from: u64,
	) -> NeoRequest<Nep17Transfers>;

	async fn get_nep17_transfers_range(
		&self,
		script_hash: H160,
		from: u64,
		to: u64,
	) -> NeoRequest<Nep17Transfers>;

	async fn get_nep11_balances(&self, script_hash: H160) -> NeoRequest<Nep11Balances>;

	async fn get_nep11_transfers(&self, script_hash: H160) -> NeoRequest<Nep11Transfers>;

	async fn get_nep11_transfers_from(
		&self,
		script_hash: H160,
		from: u64,
	) -> NeoRequest<Nep11Transfers>;

	async fn get_nep11_transfers_range(
		&self,
		script_hash: H160,
		from: u64,
		to: u64,
	) -> NeoRequest<Nep11Transfers>;

	async fn get_nep11_properties(
		&self,
		script_hash: H160,
		token_id: &str,
	) -> NeoRequest<HashMap<String, String>>;

	// StateService Methods

	async fn get_state_root(&self, index: i32) -> NeoRequest<StateRoot>;

	async fn get_proof(
		&self,
		root_hash: H256,
		contract_hash: H160,
		key: &str,
	) -> NeoRequest<String>;

	async fn verify_proof(&self, root_hash: H256, proof: &str) -> NeoRequest<bool>;

	async fn get_state_height(&self) -> NeoRequest<StateHeight>;

	async fn get_state(
		&self,
		root_hash: H256,
		contract_hash: H160,
		key: &str,
	) -> NeoRequest<String>;

	async fn find_states(
		&self,
		root_hash: H256,
		contract_hash: H160,
		prefix: &str,
		start_key: Option<&str>,
		count: Option<i32>,
	) -> NeoRequest<States>;

	// Additional Blockchain Methods

	async fn get_block_by_index(&self, index: i32, full_tx: bool) -> NeoRequest<NeoBlock>;

	async fn get_raw_block_by_index(&self, index: i32) -> NeoRequest<String>;

	// Additional Smart Contract Methods

	async fn invoke_function_diagnostics(
		&self,
		contract_hash: H160,
		name: String,
		params: Vec<ContractParameter>,
		signers: Vec<Box<dyn Signer>>,
	) -> NeoRequest<InvocationResult>;

	async fn invoke_script_diagnostics(
		&self,
		hex: String,
		signers: Vec<Box<dyn Signer>>,
	) -> NeoRequest<InvocationResult>;

	async fn traverse_iterator(
		&self,
		session_id: String,
		iterator_id: String,
		count: i32,
	) -> NeoRequest<Vec<StackItem>>;

	async fn terminate_session(&self, session_id: String) -> NeoRequest<bool>;

	async fn invoke_contract_verify(
		&self,
		contract_hash: H160,
		params: Vec<ContractParameter>,
		signers: Vec<Box<dyn Signer>>,
	) -> NeoRequest<InvocationResult>;

	// Additional Wallet Methods

	async fn get_raw_mempool(&self) -> NeoRequest<MemPoolDetails>;
	async fn import_private_key(&self, wif: String) -> NeoRequest<NeoAddress>;
	async fn get_block_header_hash(&self, hash: H256) -> NeoRequest<NeoBlock>;
	async fn send_to_address_send_token(
		&self,
		send_token: &TransactionSendToken,
	) -> NeoRequest<Transaction>;
	async fn send_from_send_token(
		&self,
		send_token: &TransactionSendToken,
		from: H160,
	) -> NeoRequest<Transaction>;
}
