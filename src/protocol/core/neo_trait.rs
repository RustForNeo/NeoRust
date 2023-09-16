use std::collections::HashMap;
use std::error::Error;
use std::vec;
use primitive_types::{H160, H256};
use crate::protocol::core::request::NeoRequest;
use crate::protocol::core::responses::contract_state::ContractState;
use crate::protocol::core::responses::invocation_result::InvocationResult;
use crate::protocol::core::responses::native_contract_state::NativeContractState;
use crate::protocol::core::responses::neo_address::NeoAddress;
use crate::protocol::core::responses::neo_application_log::NeoApplicationLog;
use crate::protocol::core::responses::neo_block::NeoBlock;
use crate::protocol::core::responses::neo_find_states::{NeoFindStates, States};
use crate::protocol::core::responses::neo_get_mem_pool::{MemPoolDetails, NeoGetMemPool};
use crate::protocol::core::responses::neo_get_nep11balances::{NeoGetNep11Balances, Nep11Balances};
use crate::protocol::core::responses::neo_get_nep11transfers::{NeoGetNep11Transfers, Nep11Transfers};
use crate::protocol::core::responses::neo_get_nep17balances::{NeoGetNep17Balances, Nep17Balances};
use crate::protocol::core::responses::neo_get_nep17transfers::{NeoGetNep17Transfers, Nep17Transfers};
use crate::protocol::core::responses::neo_get_next_block_validators::{NeoGetNextBlockValidators, Validator};
use crate::protocol::core::responses::neo_get_peers::{NeoGetPeers, Peers};
use crate::protocol::core::responses::neo_get_state_height::{NeoGetStateHeight, StateHeight};
use crate::protocol::core::responses::neo_get_state_root::{NeoGetStateRoot, StateRoot};
use crate::protocol::core::responses::neo_get_unclaimed_gas::{GetUnclaimedGas, NeoGetUnclaimedGas};
use crate::protocol::core::responses::neo_get_version::{NeoGetVersion, NeoVersion};
use crate::protocol::core::responses::neo_get_wallet_balance::{Balance, NeoGetWalletBalance};
use crate::protocol::core::responses::neo_list_plugins::{NeoListPlugins, Plugin};
use crate::protocol::core::responses::neo_network_fee::NeoNetworkFee;
use crate::protocol::core::responses::neo_response_aliases::{NeoBlockCount, NeoBlockHash, NeoBlockHeaderCount, NeoCalculateNetworkFee, NeoCloseWallet, NeoConnectionCount, NeoDumpPrivKey, NeoGetApplicationLog, NeoGetBlock, NeoGetCommittee, NeoGetContractState, NeoGetNativeContracts, NeoGetNep11Properties, NeoGetNewAddress, NeoGetProof, NeoGetRawBlock, NeoGetRawMemPool, NeoGetRawTransaction, NeoGetState, NeoGetStorage, NeoGetTransaction, NeoGetTransactionHeight, NeoGetWalletUnclaimedGas, NeoImportPrivKey, NeoInvokeContractVerify, NeoInvokeFunction, NeoInvokeScript, NeoListAddress, NeoOpenWallet, NeoSendFrom, NeoSendMany, NeoSendToAddress, NeoSubmitBlock, NeoTerminateSession, NeoTraverseIterator, NeoVerifyProof};
use crate::protocol::core::responses::neo_send_raw_transaction::NeoSendRawTransaction;
use crate::protocol::core::responses::neo_validate_address::NeoValidateAddress;
use crate::protocol::core::responses::transaction::Transaction;
use crate::protocol::core::responses::transaction_send_token::TransactionSendToken;
use crate::protocol::core::stack_item::StackItem;
use crate::transaction::signer::Signer;
use crate::types::contract_parameter::ContractParameter;

pub trait NeoTrait<T> where T: Signer {

        // Blockchain Methods

        fn get_best_block_hash(&self) -> NeoRequest<NeoBlockHash, H256>;

        fn get_block_hash(&self, block_index: i32) -> NeoRequest<NeoBlockHash, H256>;

        fn get_block(&self, block_hash: H256, return_full_tx: bool) -> NeoRequest<NeoGetBlock, NeoBlock>;

        fn get_raw_block(&self, block_hash: H256) -> NeoRequest<NeoGetRawBlock, String>;

        fn get_block_header_count(&self) -> NeoRequest<NeoBlockHeaderCount, i32>;

        fn get_block_count(&self) -> NeoRequest<NeoBlockCount, i32>;

        fn get_block_header(&self, block_hash: H256) -> NeoRequest<NeoGetBlock, NeoBlock>;

        fn get_block_header_by_index(&self, index: i32) -> NeoRequest<NeoGetBlock, NeoBlock>;

        fn get_raw_block_header(&self, block_hash: H256) -> NeoRequest<NeoGetRawBlock, String>;

        fn get_raw_block_header_by_index(&self, index: i32) -> NeoRequest<NeoGetRawBlock, String>;

        fn get_native_contracts(&self) -> NeoRequest<NeoGetNativeContracts, Vec<NativeContractState>>;

        fn get_contract_state(&self, contract_hash: H160) -> NeoRequest<NeoGetContractState, ContractState>;

        fn get_native_contract_state(&self, name: &str) -> NeoRequest<NeoGetContractState, ContractState>;

        fn get_mem_pool(&self) -> NeoRequest<NeoGetMemPool, MemPoolDetails>;

        fn get_raw_mem_pool(&self) -> NeoRequest<NeoGetRawMemPool, Vec<H256>>;

        fn get_transaction(&self, tx_hash: H256) -> NeoRequest<NeoGetTransaction, Transaction>;

        fn get_raw_transaction(&self, tx_hash: H256) -> NeoRequest<NeoGetRawTransaction, String>;

        fn get_storage(&self, contract_hash: H160, key: &str) -> NeoRequest<NeoGetStorage, String>;

        fn get_transaction_height(&self, tx_hash: H256) -> NeoRequest<NeoGetTransactionHeight, i32>;

        fn get_next_block_validators(&self) -> NeoRequest<NeoGetNextBlockValidators, Vec<Validator>>;

        fn get_committee(&self) -> NeoRequest<NeoGetCommittee, Vec<String>>;


        // Node Methods

        fn get_connection_count(&self) -> NeoRequest<NeoConnectionCount, i32>;

        fn get_peers(&self) -> NeoRequest<NeoGetPeers, Peers>;

        fn get_version(&self) -> NeoRequest<NeoGetVersion, NeoVersion>;

        fn send_raw_transaction(&self, hex: String) -> NeoRequest<NeoSendRawTransaction, RawTxResult>;

        fn submit_block(&self, hex: String) -> NeoRequest<NeoSubmitBlock, bool>;


        // Smart Contract Methods

        fn invoke_function(&self, contract_hash: H160, name: String, params: Vec<ContractParameter>, signers: Vec<T>) -> NeoRequest<NeoInvokeFunction, InvocationResult>;

        fn invoke_script(&self, hex: String, signers: Vec<T>) -> NeoRequest<NeoInvokeScript, InvocationResult>;

        fn get_unclaimed_gas(&self, script_hash: H160) -> NeoRequest<NeoGetUnclaimedGas, GetUnclaimedGas>;


        // Utility Methods

        fn list_plugins(&self) -> NeoRequest<NeoListPlugins, Vec<Plugin>>;

        fn validate_address(&self, address: &str) -> NeoRequest<NeoValidateAddress, ValidateAddressResult>;


        // Wallet Methods

        fn close_wallet(&self) -> NeoRequest<NeoCloseWallet, bool>;

        fn dump_priv_key(&self, script_hash: H160) -> NeoRequest<NeoDumpPrivKey, String>;

        fn get_wallet_balance(&self, token_hash: H160) -> NeoRequest<NeoGetWalletBalance, Balance>;

        fn get_new_address(&self) -> NeoRequest<NeoGetNewAddress, String>;

        fn get_wallet_unclaimed_gas(&self) -> NeoRequest<NeoGetWalletUnclaimedGas, String>;

        fn import_priv_key(&self, wif: String) -> NeoRequest<NeoImportPrivKey, NeoAddress>;

        fn calculate_network_fee(&self, hex: String) -> NeoRequest<NeoCalculateNetworkFee, NeoNetworkFee>;

        fn list_address(&self) -> NeoRequest<NeoListAddress, Vec<NeoAddress>>;

        fn open_wallet(&self, path: String, password: String) -> NeoRequest<NeoOpenWallet, bool>;

        fn send_from(&self, token_hash: H160, from: H160, to: H160, amount: i32) -> NeoRequest<NeoSendFrom, Transaction>;

        fn send_many(&self, from: Option<H160>, send_tokens: Vec<TransactionSendToken>) -> NeoRequest<NeoSendMany, Transaction>

        fn send_to_address(&self, token_hash: H160, to: H160, amount: i32) -> NeoRequest<NeoSendToAddress, Transaction>;


        // Application Logs

        fn get_application_log(&self, tx_hash: H256) -> NeoRequest<NeoGetApplicationLog, NeoApplicationLog>;


        // TokenTracker Methods

        fn get_nep17_balances(&self, script_hash: H160) -> NeoRequest<NeoGetNep17Balances, Nep17Balances>;

        fn get_nep17_transfers(&self, script_hash: H160) -> NeoRequest<NeoGetNep17Transfers, Nep17Transfers>;

        fn get_nep17_transfers_from(&self, script_hash: H160, from: u64) -> NeoRequest<NeoGetNep17Transfers, Nep17Transfers>;

        fn get_nep17_transfers_range(&self, script_hash: H160, from: u64, to: u64) -> NeoRequest<NeoGetNep17Transfers, Nep17Transfers>;

        fn get_nep11_balances(&self, script_hash: H160) -> NeoRequest<NeoGetNep11Balances, Nep11Balances>;

        fn get_nep11_transfers(&self, script_hash: H160) -> NeoRequest<NeoGetNep11Transfers, Nep11Transfers>;

        fn get_nep11_transfers_from(&self, script_hash: H160, from: u64) -> NeoRequest<NeoGetNep11Transfers, Nep11Transfers>;

        fn get_nep11_transfers_range(&self, script_hash: H160, from: u64, to: u64) -> NeoRequest<NeoGetNep11Transfers, Nep11Transfers>;

        fn get_nep11_properties(&self, script_hash: H160, token_id: &str) -> NeoRequest<NeoGetNep11Properties, HashMap<String, String>>;


        // StateService Methods

        fn get_state_root(&self, index: i32) -> NeoRequest<NeoGetStateRoot, StateRoot>;

        fn get_proof(&self, root_hash: H256, contract_hash: H160, key: &str) -> NeoRequest<NeoGetProof, String>;

        fn verify_proof(&self, root_hash: H256, proof: &str) -> NeoRequest<NeoVerifyProof, bool>;

        fn get_state_height(&self) -> NeoRequest<NeoGetStateHeight, StateHeight>;

        fn get_state(&self, root_hash: H256, contract_hash: H160, key: &str) -> NeoRequest<NeoGetState, String>;

        fn find_states(&self, root_hash: H256, contract_hash: H160, prefix: &str, start_key: Option<&str>, count: Option<i32>) -> NeoRequest<NeoFindStates, States>;


        // Additional Blockchain Methods

        fn get_block_by_index(&self, index: i32, full_tx: bool) -> NeoRequest<NeoGetBlock, NeoBlock>;

        fn get_raw_block_by_index(&self, index: i32) -> NeoRequest<NeoGetRawBlock, String>;


// Additional Smart Contract Methods

        fn invoke_function_diagnostics(&self, contract_hash: H160, name: String, params: Vec<ContractParameter>, signers: Vec<T>) -> NeoRequest<NeoInvokeFunction, InvocationResult>;

        fn invoke_script_diagnostics(&self, hex: String, signers: Vec<T>) -> NeoRequest<NeoInvokeScript, InvocationResult>;

        fn traverse_iterator(&self, session_id: String, iterator_id: String, count: i32) -> NeoRequest<NeoTraverseIterator, Vec<StackItem>>;

        fn terminate_session(&self, session_id: String) -> NeoRequest<NeoTerminateSession, bool>;

        fn invoke_contract_verify(&self, contract_hash: H160, params: Vec<ContractParameter>, signers: Vec<T>) -> NeoRequest<NeoInvokeContractVerify, InvocationResult>;


// Additional Wallet Methods

        async fn get_raw_mempool(&self) -> NeoRequest<NeoGetRawMemPool,MemPoolDetails>;
        async fn import_private_key(&self, wif: String) -> NeoRequest<NeoImportPrivKey,NeoAddress>;
        async fn get_block_header_hash(&self, hash: H256) -> NeoRequest<NeoGetBlock,NeoBlock>;
        async fn send_to_address_send_token(&self, send_token: &TransactionSendToken) -> NeoRequest<NeoSendToAddress, Transaction>;
        async fn send_from_send_token(&self, send_token: &TransactionSendToken, from: H160) -> NeoRequest<TransactionSendToken,Transaction>;
}