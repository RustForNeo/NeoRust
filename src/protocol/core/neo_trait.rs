use std::collections::HashMap;
use std::error::Error;
use primitive_types::{H160, H256};
use crate::protocol::core::responses::contract_state::ContractState;
use crate::protocol::core::responses::invocation_result::InvocationResult;
use crate::protocol::core::responses::native_contract_state::NativeContractState;
use crate::protocol::core::responses::neo_application_log::NeoApplicationLog;
use crate::protocol::core::responses::neo_block::NeoBlock;
use crate::protocol::core::responses::neo_find_states::{NeoFindStates, States};
use crate::protocol::core::responses::neo_get_nep11balances::Nep11Balances;
use crate::protocol::core::responses::neo_get_nep11transfers::Nep11Transfers;
use crate::protocol::core::responses::neo_get_nep17balances::Nep17Balances;
use crate::protocol::core::responses::neo_get_nep17transfers::Nep17Transfers;
use crate::protocol::core::responses::neo_get_next_block_validators::Validator;
use crate::protocol::core::responses::neo_get_peers::Peers;
use crate::protocol::core::responses::neo_get_state_height::StateHeight;
use crate::protocol::core::responses::neo_get_state_root::StateRoot;
use crate::protocol::core::responses::neo_get_unclaimed_gas::NeoGetUnclaimedGas;
use crate::protocol::core::responses::neo_get_version::NeoVersion;
use crate::protocol::core::responses::neo_get_wallet_balance::Balance;
use crate::protocol::core::responses::neo_list_plugins::NeoListPlugins;
use crate::protocol::core::responses::neo_network_fee::NeoNetworkFee;
use crate::protocol::core::responses::neo_response_aliases::{NeoGetRawMemPool, NeoTerminateSession};
use crate::protocol::core::responses::neo_send_raw_transaction::RawTransaction;
use crate::protocol::core::responses::neo_validate_address::NeoValidateAddress;
use crate::protocol::core::responses::transaction::Transaction;
use crate::protocol::core::responses::transaction_send_token::TransactionSendToken;
use crate::protocol::neo_config::NeoConfig;
use crate::protocol::protocol_error::ProtocolError;
use crate::transaction::signer::Signer;
use crate::types::{Address, Bytes};
use crate::types::contract_parameter::ContractParameter;

pub trait Neo {

    fn config(&self) -> &NeoConfig;

    fn nns_resolver(&self) -> H160;
    fn block_interval(&self) -> u32;
    fn polling_interval(&self) -> u32;
    fn max_valid_until_block_increment(&self) -> u32;

    fn get_network_magic_number_bytes(&self) -> Bytes;
    fn get_network_magic_number(&self) -> Result<u32, Box<dyn Error>>;

    // Blockchain methods
    fn get_best_block_hash(&self) -> H256;
    fn get_block_hash(&self, block_index: u32) -> H256;
    fn get_block(&self, block_hash: H256, return_full_tx: bool) -> Result<NeoBlock, Box<dyn Error>>;
    fn get_raw_block(&self, block_hash: H256) -> Result<String, Box<dyn Error>>;
    fn get_block_header(&self, block_hash: H256) -> Result<NeoBlock, Box<dyn Error>>;
    fn get_block_header_count(&self) -> Result<u32, Box<dyn Error>>;
    fn get_block_count(&self) -> Result<u32, Box<dyn Error>>;
    fn get_native_contracts(&self) -> Result<Vec<NativeContractState>, Box<dyn Error>>;
    fn get_contract_state(&self, contract_hash: H160) -> Result<ContractState, Box<dyn Error>>;
    fn get_storage(&self, contract_hash: H160, key: String) -> Result<String, Box<dyn Error>>;
    fn get_transaction(&self, tx_hash: H256) -> Result<Transaction, Box<dyn Error>>;
    fn get_transaction_height(&self, tx_hash: H256) -> Result<u32, Box<dyn Error>>;
    fn get_next_block_validators(&self) -> Result<Vec<Validator>, Box<dyn Error>>;
    fn get_committee(&self) -> Result<Vec<String>, Box<dyn Error>>;

    // Node methods
    fn get_connection_count(&self) -> Result<u32, Box<dyn Error>>;
    fn get_peers(&self) -> Result<Peers, Box<dyn Error>>;
    fn get_version(&self) -> Result<NeoVersion, Box<dyn Error>>;
    fn send_raw_transaction(&self, raw_tx: String) -> Result<RawTransaction, Box<dyn Error>>;
    fn submit_block(&self, serialized_block: String) -> Result<bool, Box<dyn Error>>;

    // Smart contract methods
    fn invoke_function(&self, contract_hash: H160, function_name: String, params: Vec<ContractParameter>, signers: Vec<dyn Signer>) -> Result<InvocationResult, Box<dyn Error>>;
    fn invoke_script(&self, script: String, signers: Vec<dyn Signer>) -> Result<InvocationResult, Box<dyn Error>>;
    fn invoke_contract_verify(&self, contract_hash: H160, params: Vec<ContractParameter>, signers: Vec<dyn Signer>) -> Result<InvocationResult, Box<dyn Error>>;
    fn get_unclaimed_gas(&self, script_hash: H160) -> Result<NeoGetUnclaimedGas, Box<dyn Error>>;

    // Utility methods
    fn validate_address(&self, address: String) -> Result<NeoValidateAddress, Box<dyn Error>>;

    // Wallet methods
    fn close_wallet(&self) -> Result<bool, Box<dyn Error>>;
    fn dump_priv_key(&self, script_hash: H160) -> Result<String, Box<dyn Error>>;
    fn get_wallet_balance(&self, token_hash: H160) -> Result<Balance, Box<dyn Error>>;
    fn get_new_address(&self) -> Result<String, Box<dyn Error>>;
    fn import_priv_key(&self, priv_key: String) -> Result<Address, Box<dyn Error>>;
    fn calculate_network_fee(&self, raw_tx: String) -> Result<NeoNetworkFee, Box<dyn Error>>;
    fn list_address(&self) -> Result<Vec<Address>, Box<dyn Error>>;
    fn open_wallet(&self, path: String, password: String) -> Result<bool, Box<dyn Error>>;
    fn send_from(&self, token_hash: H160, from: H160, to: H160, amount: u32) -> Result<Transaction, Box<dyn Error>>;
    fn send_from_send_token(&self, send_token: &TransactionSendToken, from: H160) -> Result<Transaction, Box<dyn Error>>;
    fn send_many(&self, from: Option<H160>, send_tokens: Vec<TransactionSendToken>) -> Result<Transaction, Box<dyn Error>>;
    fn send_to_address(&self, token_hash: H160, to: H160, amount: u32) -> Result<Transaction, Box<dyn Error>>;
    fn send_to_address_send_token(&self, send_token: &TransactionSendToken) -> Result<Transaction, Box<dyn Error>>;

    // Application logs
    fn get_application_log(&self, tx_hash: H256) -> Result<NeoApplicationLog, Box<dyn Error>>;

    // NEP-17 methods
    fn get_nep17_balances(&self, script_hash: H160) -> Result<Nep17Balances, Box<dyn Error>>;
    fn get_nep17_transfers(&self, script_hash: H160) -> Result<Nep17Transfers, Box<dyn Error>>;
    fn get_nep17_transfers_from(&self, script_hash: H160, from: i64) -> Result<Nep17Transfers, Box<dyn Error>>;
    fn get_nep17_transfers_range(&self, script_hash: H160, from: i64, to: i64) -> Result<Nep17Transfers, Box<dyn Error>>;

    // NEP-11 methods
    fn get_nep11_balances(&self, script_hash: H160) -> Result<Nep11Balances, Box<dyn Error>>;
    fn get_nep11_transfers(&self, script_hash: H160) -> Result<Nep11Transfers, Box<dyn Error>>;
    fn get_nep11_transfers_from(&self, script_hash: H160, from: i64) -> Result<Nep11Transfers, Box<dyn Error>>;
    fn get_nep11_transfers_range(&self, script_hash: H160, from: i64, to: i64) -> Result<Nep11Transfers, Box<dyn Error>>;
    fn get_nep11_properties(&self, script_hash: H160, token_id: String) -> Result<HashMap<String, String>, Box<dyn Error>>;

    // State service methods
    fn get_state_root(&self, block_index: u32) -> Result<StateRoot, Box<dyn Error>>;
    fn get_proof(&self, root_hash: H256, contract_hash: H160, storage_key: String) -> Result<String, Box<dyn Error>>;
    fn verify_proof(&self, root_hash: H256, proof: String) -> Result<bool, Box<dyn Error>>;
    fn get_state_height(&self) -> Result<StateHeight, Box<dyn Error>>;
    fn get_state(&self, root_hash: H256, contract_hash: H160, key: String) -> Result<String, Box<dyn Error>>;
    fn find_states(&self, root_hash: H256, contract_hash: H160, key_prefix: String, start_key: Option<String>, count: Option<u32>) -> Result<States, Box<dyn Error>>;
    async fn list_plugins(&self) -> Result<NeoListPlugins, ProtocolError>;
    async fn get_raw_mempool(&self) -> Result<NeoGetRawMemPool, ProtocolError>;
    async fn terminate_session(&self, session_id: &String) -> Result<NeoTerminateSession, ProtocolError>;
}