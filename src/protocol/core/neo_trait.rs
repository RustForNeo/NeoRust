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
	fn get_best_block_hash(&self) -> NeoRequest<H256>;

	fn get_block_hash(&self, block_index: i32) -> NeoRequest<H256>;

	fn get_block(&self, block_hash: H256, return_full_tx: bool) -> NeoRequest<NeoBlock>;

	fn get_raw_block(&self, block_hash: H256) -> NeoRequest<String>;

	fn get_block_header_count(&self) -> NeoRequest<i32>;

	fn get_block_count(&self) -> NeoRequest<i32>;

	fn get_block_header(&self, block_hash: H256) -> NeoRequest<NeoBlock>;

	fn get_block_header_by_index(&self, index: i32) -> NeoRequest<NeoBlock>;

	fn get_raw_block_header(&self, block_hash: H256) -> NeoRequest<String>;

	fn get_raw_block_header_by_index(&self, index: i32) -> NeoRequest<String>;

	fn get_native_contracts(&self) -> NeoRequest<Vec<NativeContractState>>;

	fn get_contract_state(&self, contract_hash: H160) -> NeoRequest<ContractState>;

	fn get_native_contract_state(&self, name: &str) -> NeoRequest<ContractState>;

	fn get_mem_pool(&self) -> NeoRequest<MemPoolDetails>;

	fn get_raw_mem_pool(&self) -> NeoRequest<Vec<H256>>;

	fn get_transaction(&self, tx_hash: H256) -> NeoRequest<Transaction>;

	fn get_raw_transaction(&self, tx_hash: H256) -> NeoRequest<String>;

	fn get_storage(&self, contract_hash: H160, key: &str) -> NeoRequest<String>;

	fn get_transaction_height(&self, tx_hash: H256) -> NeoRequest<i32>;

	fn get_next_block_validators(&self) -> NeoRequest<Vec<Validator>>;

	fn get_committee(&self) -> NeoRequest<Vec<String>>;

	// Node Methods

	fn get_connection_count(&self) -> NeoRequest<i32>;

	fn get_peers(&self) -> NeoRequest<Peers>;

	fn get_version(&self) -> NeoRequest<NeoVersion>;

	fn send_raw_transaction(&self, hex: String) -> NeoRequest<RawTransaction>;

	fn submit_block(&self, hex: String) -> NeoRequest<bool>;

	// Smart Contract Methods

	fn invoke_function(
		&self,
		contract_hash: &H160,
		name: String,
		params: Vec<ContractParameter>,
		signers: Vec<Box<dyn Signer>>,
	) -> NeoRequest<InvocationResult>;

	fn invoke_script(
		&self,
		hex: String,
		signers: Vec<Box<dyn Signer>>,
	) -> NeoRequest<InvocationResult>;

	fn get_unclaimed_gas(&self, script_hash: H160) -> NeoRequest<GetUnclaimedGas>;

	// Utility Methods

	fn list_plugins(&self) -> NeoRequest<Vec<Plugin>>;

	fn validate_address(&self, address: &str) -> NeoRequest<ValidateAddress>;

	// Wallet Methods

	fn close_wallet(&self) -> NeoRequest<bool>;

	fn dump_priv_key(&self, script_hash: H160) -> NeoRequest<String>;

	fn get_wallet_balance(&self, token_hash: H160) -> NeoRequest<Balance>;

	fn get_new_address(&self) -> NeoRequest<String>;

	fn get_wallet_unclaimed_gas(&self) -> NeoRequest<String>;

	fn import_priv_key(&self, wif: String) -> NeoRequest<NeoAddress>;

	fn calculate_network_fee(&self, hex: String) -> NeoRequest<NeoNetworkFee>;

	fn list_address(&self) -> NeoRequest<Vec<NeoAddress>>;

	fn open_wallet(&self, path: String, password: String) -> NeoRequest<bool>;

	fn send_from(
		&self,
		token_hash: H160,
		from: H160,
		to: H160,
		amount: i32,
	) -> NeoRequest<Transaction>;

	fn send_many(
		&self,
		from: Option<H160>,
		send_tokens: Vec<TransactionSendToken>,
	) -> NeoRequest<Transaction>;

	fn send_to_address(&self, token_hash: H160, to: H160, amount: i32) -> NeoRequest<Transaction>;

	// Application Logs

	fn get_application_log(&self, tx_hash: H256) -> NeoRequest<NeoApplicationLog>;

	// TokenTracker Methods

	fn get_nep17_balances(&self, script_hash: H160) -> NeoRequest<Nep17Balances>;

	fn get_nep17_transfers(&self, script_hash: H160) -> NeoRequest<Nep17Transfers>;

	fn get_nep17_transfers_from(&self, script_hash: H160, from: u64) -> NeoRequest<Nep17Transfers>;

	fn get_nep17_transfers_range(
		&self,
		script_hash: H160,
		from: u64,
		to: u64,
	) -> NeoRequest<Nep17Transfers>;

	fn get_nep11_balances(&self, script_hash: H160) -> NeoRequest<Nep11Balances>;

	fn get_nep11_transfers(&self, script_hash: H160) -> NeoRequest<Nep11Transfers>;

	fn get_nep11_transfers_from(&self, script_hash: H160, from: u64) -> NeoRequest<Nep11Transfers>;

	fn get_nep11_transfers_range(
		&self,
		script_hash: H160,
		from: u64,
		to: u64,
	) -> NeoRequest<Nep11Transfers>;

	fn get_nep11_properties(
		&self,
		script_hash: H160,
		token_id: &str,
	) -> NeoRequest<HashMap<String, String>>;

	// StateService Methods

	fn get_state_root(&self, index: i32) -> NeoRequest<StateRoot>;

	fn get_proof(&self, root_hash: H256, contract_hash: H160, key: &str) -> NeoRequest<String>;

	fn verify_proof(&self, root_hash: H256, proof: &str) -> NeoRequest<bool>;

	fn get_state_height(&self) -> NeoRequest<StateHeight>;

	fn get_state(&self, root_hash: H256, contract_hash: H160, key: &str) -> NeoRequest<String>;

	fn find_states(
		&self,
		root_hash: H256,
		contract_hash: H160,
		prefix: &str,
		start_key: Option<&str>,
		count: Option<i32>,
	) -> NeoRequest<States>;

	// Additional Blockchain Methods

	fn get_block_by_index(&self, index: i32, full_tx: bool) -> NeoRequest<NeoBlock>;

	fn get_raw_block_by_index(&self, index: i32) -> NeoRequest<String>;

	// Additional Smart Contract Methods

	fn invoke_function_diagnostics(
		&self,
		contract_hash: H160,
		name: String,
		params: Vec<ContractParameter>,
		signers: Vec<Box<dyn Signer>>,
	) -> NeoRequest<InvocationResult>;

	fn invoke_script_diagnostics(
		&self,
		hex: String,
		signers: Vec<Box<dyn Signer>>,
	) -> NeoRequest<InvocationResult>;

	fn traverse_iterator(
		&self,
		session_id: String,
		iterator_id: String,
		count: i32,
	) -> NeoRequest<Vec<StackItem>>;

	fn terminate_session(&self, session_id: String) -> NeoRequest<bool>;

	fn invoke_contract_verify(
		&self,
		contract_hash: H160,
		params: Vec<ContractParameter>,
		signers: Vec<Box<dyn Signer>>,
	) -> NeoRequest<InvocationResult>;

	// Additional Wallet Methods

	fn get_raw_mempool(&self) -> NeoRequest<MemPoolDetails>;
	fn import_private_key(&self, wif: String) -> NeoRequest<NeoAddress>;
	fn get_block_header_hash(&self, hash: H256) -> NeoRequest<NeoBlock>;
	fn send_to_address_send_token(
		&self,
		send_token: &TransactionSendToken,
	) -> NeoRequest<Transaction>;
	fn send_from_send_token(
		&self,
		send_token: &TransactionSendToken,
		from: H160,
	) -> NeoRequest<Transaction>;
}
