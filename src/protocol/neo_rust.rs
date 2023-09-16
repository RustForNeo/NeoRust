use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use futures::{SinkExt, Stream, stream};
use lazy_static::lazy_static;
use primitive_types::{H160, H256};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::neo_error::NeoError;
use crate::protocol::core::neo_trait::NeoTrait;
use crate::protocol::core::request::NeoRequest;
use crate::protocol::core::responses::contract_state::ContractState;
use crate::protocol::core::responses::invocation_result::InvocationResult;
use crate::protocol::core::responses::native_contract_state::NativeContractState;
use crate::protocol::core::responses::neo_address::NeoAddress;
use crate::protocol::core::responses::neo_application_log::NeoApplicationLog;
use crate::protocol::core::responses::neo_block::NeoBlock;
use crate::protocol::core::responses::neo_find_states::{NeoFindStates, States};
use crate::protocol::core::responses::neo_get_mem_pool::{MemPoolDetails, NeoGetMemPool};
use crate::protocol::core::responses::neo_get_nep11balances::Nep11Balances;
use crate::protocol::core::responses::neo_get_nep11transfers::{NeoGetNep11Transfers, Nep11Transfers};
use crate::protocol::core::responses::neo_get_nep17balances::{NeoGetNep17Balances, Nep17Balances};
use crate::protocol::core::responses::neo_get_nep17transfers::{NeoGetNep17Transfers, Nep17Transfers};
use crate::protocol::core::responses::neo_get_next_block_validators::{NeoGetNextBlockValidators, Validator};
use crate::protocol::core::responses::neo_get_peers::{NeoGetPeers, Peers};
use crate::protocol::core::responses::neo_get_state_height::{NeoGetStateHeight, StateHeight};
use crate::protocol::core::responses::neo_get_state_root::{NeoGetStateRoot, StateRoot};
use crate::protocol::core::responses::neo_get_token_balances::TokenBalance;
use crate::protocol::core::responses::neo_get_unclaimed_gas::{GetUnclaimedGas, NeoGetUnclaimedGas};
use crate::protocol::core::responses::neo_get_version::{NeoGetVersion, NeoVersion};
use crate::protocol::core::responses::neo_get_wallet_balance::NeoGetWalletBalance;
use crate::protocol::core::responses::neo_list_plugins::{NeoListPlugins, Plugin};
use crate::protocol::core::responses::neo_network_fee::NeoNetworkFee;
use crate::protocol::core::responses::neo_response_aliases::{NeoBlockCount, NeoBlockHash, NeoBlockHeaderCount, NeoCalculateNetworkFee, NeoCloseWallet, NeoConnectionCount, NeoDumpPrivKey, NeoGetApplicationLog, NeoGetBlock, NeoGetCommittee, NeoGetContractState, NeoGetNativeContracts, NeoGetNep11Properties, NeoGetNewAddress, NeoGetProof, NeoGetRawBlock, NeoGetRawMemPool, NeoGetRawTransaction, NeoGetState, NeoGetStorage, NeoGetTransaction, NeoGetTransactionHeight, NeoGetWalletUnclaimedGas, NeoImportPrivKey, NeoInvokeContractVerify, NeoInvokeFunction, NeoInvokeScript, NeoListAddress, NeoOpenWallet, NeoSendFrom, NeoSendMany, NeoSendToAddress, NeoSubmitBlock, NeoTerminateSession, NeoTraverseIterator, NeoVerifyProof};
use crate::protocol::core::responses::neo_send_raw_transaction::{NeoSendRawTransaction, RawTransaction};
use crate::protocol::core::responses::neo_validate_address::NeoValidateAddress;
use crate::protocol::core::responses::transaction::Transaction;
use crate::protocol::core::responses::transaction_send_token::TransactionSendToken;
use crate::protocol::core::responses::transaction_signer::TransactionSigner;
use crate::protocol::core::stack_item::StackItem;
use crate::protocol::http_service::HttpService;
use crate::protocol::neo_config::NeoConfig;
use crate::protocol::neo_service::NeoService;
use crate::transaction::signer::Signer;
use crate::types::{Address, Bytes, ExternBase64, H160Externsion, ValueExtension};
use crate::types::contract_parameter::ContractParameter;
use crate::utils::bytes::BytesExtern;

lazy_static! {
  pub static ref NEO_RUST_INSTANCE: Mutex<NeoRust> =
       Mutex::new(NeoRust::new());
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NeoRust {
    config: Arc<NeoConfig>,
    neo_service: Arc<Mutex< dyn NeoService>>,
}

impl NeoRust{
    pub fn new() -> Self{

        Self {
            config: Arc::new(NeoConfig::default()),
            neo_service: Arc::new(Mutex::new(HttpService::new(Url::from_str("").unwrap(), false))),
        }
    }
    pub fn instance() -> MutexGuard<'static, Self> {
        NEO_RUST_INSTANCE.lock().unwrap()
    }


    fn config(&self) -> &NeoConfig {
        &self.config
    }

    fn nns_resolver(&self) -> H160 {
        H160::from(self.config.nns_resolver.clone())
    }

    fn block_interval(&self) -> u32 {
        self.config.block_interval as u32
    }

    fn polling_interval(&self) -> u32 {
        self.config.polling_interval as u32
    }

    fn max_valid_until_block_increment(&self) -> u32 {
        self.config.max_valid_until_block_increment as u32
    }

    pub(crate) fn get_neo_service(&self) -> &dyn NeoService {
        &self.neo_service.lock().unwrap()
    }

    fn get_neo_service_mut(&mut self) -> &mut dyn NeoService {
        &mut self.neo_service
    }

    async fn dump_private_key(&self, script_hash: H160) -> NeoRequest<NeoDumpPrivKey,String> {
        NeoRequest::new("dumpprivkey", vec![Value::String(script_hash.to_address())])
    }

    pub async fn block_publisher(&self, full_tx: bool) -> NeoRequest<,impl Stream<Item = Result<NeoBlock, NeoError>>> {
        let interval = Duration::from_millis(self.config.polling_interval);

        let stream = stream::unfold((), move |_| async {
            let block_hash = self.get_best_block_hash().send(()).await?.result;
            let block = self.get_block(block_hash, full_tx).await?;
            Some((block, ()))
        })
            .throttle(interval);

        Ok(stream)
    }

    async fn get_network_magic_number(&mut self) -> Result<i32, NeoError> {
        if self.config.network_magic.is_none() {
            let magic = self.get_version()
                .await?
                .get_result()
                .protocol
                .ok_or(NeoError::IllegalState("Unable to read Network Magic Number from Version".to_string()))?
                .network;
            self.config.set_network_magic(magic).expect("Unable to set Network Magic Number");
        }
        Ok(self.config.network_magic.unwrap() as i32)
    }

    async fn get_network_magic_number_bytes(&mut self) -> Result<Bytes, NeoError> {
        let magic_int = self.get_network_magic_number().send().await? & 0xFFFF_FFFF;
        Ok(magic_int.to_be_bytes().to_vec())
    }
}

impl<T> NeoTrait<T> for NeoRust
where T: Signer+Serialize+Deserialize{

    // Blockchain methods
    async fn get_best_block_hash(&self) -> NeoRequest<NeoBlockHash,H256> {
        NeoRequest::new("getbestblockhash", vec![])
    }

    async fn get_block_hash(&self, block_index: i32) -> NeoRequest<NeoBlockHash,H256> {
        NeoRequest::new("getblockhash", [block_index.to_value()].to_vec())
    }

    async fn get_block(&self, block_hash: H256, full_tx: bool) -> NeoRequest<NeoBlockHash,NeoBlock> {
        if full_tx {
             NeoRequest::new("getblock", [block_hash.to_value(), 1].to_vec())
        } else {
            self.get_block_header_hash(block_hash)
        }
    }

    // More methods...

    async fn get_raw_block(&self, block_hash: H256) -> NeoRequest<NeoGetRawBlock,String> {
        NeoRequest::new("getblock", vec![block_hash.to_value(), 0])
   }

    // Node methods

    async fn get_block_header_count(&self) -> NeoRequest<NeoBlockHeaderCount,u32> {
         NeoRequest::new("getblockheadercount", vec![])
    }

    async fn get_block_count(&self) -> NeoRequest<NeoBlockCount,i32> {
        NeoRequest::new("getblockcount", vec![])
    }

    async fn get_block_header(&self, index: i32) -> NeoRequest<NeoGetBlock,NeoBlock> {
        NeoRequest::new("getblockheader", vec![index.to_value(), 1])
   }

    fn get_block_header_by_index(&self, index: i32) -> NeoRequest<NeoGetBlock, NeoBlock> {
        NeoRequest::new("getblockheader",
            vec![index.to_value(), 1.to_value()],
        )
    }

// Smart contract methods

    fn get_raw_block_header(&self, block_hash: H256) -> NeoRequest<NeoGetRawBlock, String> {
        NeoRequest::new("getblockheader",
            vec![block_hash.to_value(), 0.to_value()],

        )
    }

    fn get_raw_block_header_by_index(&self, index: i32) -> NeoRequest<NeoGetRawBlock, String> {
        NeoRequest::new("getblockheader",
            vec![index.to_value(), 0.to_value()],

        )
    }

// Utility methods

    async fn get_native_contracts(&self) -> NeoRequest<NeoGetNativeContracts,Vec<NativeContractState>> {
         NeoRequest::new("getnativecontracts", vec![])
    }

// Wallet methods

    async fn get_contract_state(&self, hash: H160) -> NeoRequest<NeoGetContractState,ContractState> {
        NeoRequest::new("getcontractstate", vec![hash.to_value()])
   }


    fn get_native_contract_state(&self, name: &str) -> NeoRequest<NeoGetContractState, ContractState> {
        NeoRequest::new("getcontractstate",
            vec![name.to_value()],

        )
    }

    fn get_mem_pool(&self) -> NeoRequest<NeoGetMemPool, MemPoolDetails> {
        NeoRequest::new("getrawmempool",
            vec![1.to_value()],

        )
    }

    fn get_raw_mem_pool(&self) -> NeoRequest<NeoGetRawMemPool, Vec<H256>> {
        NeoRequest::new("getrawmempool",
            vec![],

        )
    }

// Application logs

    async fn get_transaction(&self, hash: H256) -> NeoRequest<NeoGetTransaction,Transaction> {
        NeoRequest::new("getrawtransaction", vec![hash.to_value(), 1])
   }

// State service

    fn get_raw_transaction(&self, tx_hash: H256) -> NeoRequest<NeoGetRawTransaction, String> {
        NeoRequest::new("getrawtransaction",
            vec![tx_hash.to_value(), 0.to_value()],

        )
    }

    async fn get_storage(&self, contract_hash: H160, key: String) -> NeoRequest<NeoGetStorage,String> {
        let params = [contract_hash.to_value(), key.to_value()];
         NeoRequest::new("getstorage", params.to_vec())
    }
// Blockchain methods

    async fn get_transaction_height(&self, tx_hash: H256) -> NeoRequest<NeoGetTransactionHeight, u32> {
        let params = [tx_hash.to_value()];
         NeoRequest::new("gettransactionheight", params.to_vec())
    }

    async fn get_next_block_validators(&self) -> NeoRequest<NeoGetNextBlockValidators,Vec<Validator>> {
          NeoRequest::new("getnextblockvalidators", vec![])
    }

    async fn get_committee(&self) -> NeoRequest<NeoGetCommittee,Vec<String>> {
        NeoRequest::new("getcommittee", vec![])
    }

    async fn get_connection_count(&self) -> NeoRequest<NeoConnectionCount,i32> {
        NeoRequest::new("getconnectioncount", vec![])
    }

    async fn get_peers(&self) -> NeoRequest<NeoGetPeers,Peers> {
            NeoRequest::new("getpeers", vec![])
    }

// Smart contract methods

    async fn get_version(&self) -> NeoRequest<NeoGetVersion,NeoVersion> {
        NeoRequest::new("getversion", vec![])
    }

    async fn send_raw_transaction(&self, hex: String) -> NeoRequest<NeoSendRawTransaction,RawTransaction> {
        NeoRequest::new("sendrawtransaction", vec![hex.to_value()])
    }
// More node methods

    async fn submit_block(&self, hex: String) -> NeoRequest<NeoSubmitBlock,bool> {
        NeoRequest::new("submitblock", vec![hex.to_value()])
    }

// More blockchain methods

    async fn invoke_function(
        &self,
        contract_hash: &H160,
        method: String,
        params: Vec<ContractParameter>,
        signers: Vec<T>,
    ) -> NeoRequest<NeoInvokeFunction,InvocationResult> {
        let signers = signers
            .into_iter()
            .map(TransactionSigner::from)
            .collect();
        NeoRequest::new("invokefunction", vec![
            contract_hash.to_value(),
            method,
            params,
            signers,
        ])
    }

    async fn invoke_script(&self, hex: String, signers: Vec<T>) -> NeoRequest<NeoInvokeScript,InvocationResult> {
        let signers = signers
            .into_iter()
            .map(TransactionSigner::from)
            .collect();

        NeoRequest::new("invokescript", vec![hex.to_value(), signers])
    }

// More smart contract methods

    async fn get_unclaimed_gas(&self, hash: H160) -> NeoRequest<NeoGetUnclaimedGas,GetUnclaimedGas> {
        NeoRequest::new("getunclaimedgas", vec![hash.to_value()])
    }

    async fn list_plugins(&self) -> NeoRequest<NeoListPlugins, Vec<Plugin>> {
        NeoRequest::new("listplugins", vec![]);

    }

// More utility methods

    async fn validate_address(&self, address: String) -> NeoRequest<NeoValidateAddress,NeoValidateAddress> {
        NeoRequest::new("validateaddress", vec![Value::String(address)])
    }

// More wallet methods

    async fn close_wallet(&self) -> NeoRequest<NeoCloseWallet,bool> {
        NeoRequest::new("closewallet", vec![])
    }

    async fn dump_priv_key(&self, script_hash: H160) -> NeoRequest<NeoDumpPrivKey,String> {
        let params = [script_hash.to_value()].to_vec();
         NeoRequest::new("dumpprivkey", params)
    }


    async fn get_wallet_balance(&self, token_hash: H160) -> NeoRequest<NeoGetWalletBalance,dyn TokenBalance> {
        NeoRequest::new("getwalletbalance", vec![token_hash.to_value()])
    }

    async fn get_new_address(&self) -> NeoRequest<NeoGetNewAddress,String> {
        NeoRequest::new("getnewaddress", vec![])
    }

    fn get_wallet_unclaimed_gas(&self) -> NeoRequest<NeoGetWalletUnclaimedGas, String> {
        NeoRequest::new("getwalletunclaimedgas",
            vec![],

        )
    }

    async fn import_priv_key(&self, priv_key: String) -> NeoRequest<NeoImportPrivKey,Address> {
        let params = [priv_key.to_value()].to_vec();
          NeoRequest::new("importprivkey", params)
    }

    async fn calculate_network_fee(&self, hex: String) -> NeoRequest<NeoCalculateNetworkFee,NeoNetworkFee> {
        NeoRequest::new("calculatenetworkfee", vec![hex.to_value()])
    }

    async fn list_address(&self) -> NeoRequest<NeoListAddress,Vec<Address>> {
          NeoRequest::new("listaddress", vec![])
    }
    async fn open_wallet(&self, path: String, password: String) -> NeoRequest<NeoOpenWallet,bool> {
        NeoRequest::new("openwallet", vec![path.to_value(), password.to_value()])
    }

    async fn send_from(&self, token_hash: H160, from: H160, to: H160, amount: u32) -> NeoRequest<NeoSendFrom,Transaction> {
        let params = [token_hash.to_value(), from.to_value(), to.to_value(), amount.to_value()].to_vec();
          NeoRequest::new("sendfrom", params)
    }

// Transaction methods

    async fn send_many(&self, from: Option<H160>, send_tokens: Vec<TransactionSendToken>) -> NeoRequest<NeoSendMany, Transaction> {
        let params = [from?.to_value(), send_tokens.to_value()].to_vec();
          NeoRequest::new("sendmany", params)
    }

    async fn send_to_address(&self, token_hash: H160, to: H160, amount: u32) -> NeoRequest<NeoSendToAddress, Transaction> {
        let params = [token_hash.to_value(), to.to_value(), amount.to_value()].to_vec();
         NeoRequest::new("sendtoaddress", params)
    }

    async fn get_application_log(&self, tx_hash: H256) -> NeoRequest<NeoGetApplicationLog,NeoApplicationLog> {
        NeoRequest::new("getapplicationlog", vec![tx_hash.to_value()])
   }

    async fn get_nep17_balances(&self, script_hash: H160) -> NeoRequest<NeoGetNep17Balances,Nep17Balances> {
         NeoRequest::new("getnep17balances", [script_hash.to_value()].to_vec())
    }

    async fn get_nep17_transfers(&self, script_hash: H160) -> NeoRequest<NeoGetNep17Transfers, Nep17Transfers> {
        let params = [script_hash.to_value()].to_vec();
         NeoRequest::new("getnep17transfers", params)
    }

// NEP-17 methods

    async fn get_nep17_transfers_from(&self, script_hash: H160, from: i64) -> NeoRequest<NeoGetNep17Transfers, Nep17Transfers> {
        let params = [script_hash.to_value(), from.to_value()].to_vec();
       NeoRequest::new("getnep17transfers", params)
    }

    async fn get_nep17_transfers_range(&self, script_hash: H160, from: i64, to: i64) -> NeoRequest<NeoGetNep17Transfers, Nep17Transfers> {
        let params = [script_hash.to_value(), from.to_value(), to.to_value()].to_vec();
         NeoRequest::new("getnep17transfers", params)
    }

    async fn get_nep11_balances(&self, script_hash: H160) -> NeoRequest<NeoGetNep11Transfers, Nep11Transfers> {
        let params = [script_hash.to_value()].to_vec();
         NeoRequest::new("getnep11balances", params)
    }

// NEP-11 methods

    async fn get_nep11_transfers(&self, script_hash: H160) ->  NeoRequest<NeoGetNep11Transfers, Nep11Transfers> {
        let params = [script_hash.to_value()].to_vec();
         NeoRequest::new("getnep11transfers", params)
    }

    async fn get_nep11_transfers_from(&self, script_hash: H160, from: i64) ->  NeoRequest<NeoGetNep11Transfers, Nep11Transfers> {
        let params = [script_hash.to_value(), from.to_value()].to_vec();
         NeoRequest::new("getnep11transfers", params)
    }

    async fn get_nep11_transfers_range(&self, script_hash: H160, from: i64, to: i64) -> NeoRequest<NeoGetNep11Transfers, Nep11Transfers> {
        let params = [script_hash.to_value(), from.to_value(), to.to_value()].to_vec();
         NeoRequest::new("getnep11transfers", params)
    }

    async fn get_nep11_properties(&self, script_hash: H160, token_id: String) -> NeoRequest<NeoGetNep11Properties, HashMap<String, String>> {
        let params = [script_hash.to_value(), token_id.to_value()].to_vec();
         NeoRequest::new("getnep11properties", params)
    }

    async fn get_state_root(&self, block_index: u32) ->  NeoRequest<NeoGetStateRoot, StateRoot> {
        let params = [block_index.to_value()].to_vec();
          NeoRequest::new("getstateroot", params)
    }

// State service methods

    async fn get_proof(&self, root_hash: H256, contract_hash: H160, key: String) -> NeoRequest<NeoGetProof,String> {
        NeoRequest::new("getproof", vec![
            root_hash.to_value(),
            contract_hash.to_value(),
            key.to_base64(),
        ])
   }

    async fn verify_proof(&self, root_hash: H256, proof: String) -> NeoRequest<NeoVerifyProof, bool> {
        let params = [root_hash.to_value(), proof.to_value()].to_vec();
        NeoRequest::new("verifyproof", params)
    }

    async fn get_state_height(&self) ->  NeoRequest<NeoGetStateHeight, StateHeight> {
         NeoRequest::new("getstateheight", vec![])
    }

    async fn get_state(&self, root_hash: H256, contract_hash: H160, key: String) -> NeoRequest<NeoGetState,String> {
        NeoRequest::new("getstate", vec![
            root_hash.to_value(),
            contract_hash.to_value(),
            key.to_base64(),
        ])
   }
    async fn find_states(&self,
                         root_hash: H256,
                         contract_hash: H160,
                         key_prefix: String,
                         start_key: Option<String>,
                         count: Option<u32>) -> NeoRequest<NeoFindStates, States> {
        let mut params = vec![root_hash.to_value(), contract_hash.to_value(), key_prefix.to_value()];
        if let Some(start_key) = start_key {
            params.push(start_key.to_value())
        }
        if let Some(count) = count {
            params.push(count.to_value())
        }

          NeoRequest::new("findstates", params)
    }

    fn get_block_by_index(&self, index: i32, full_tx: bool) -> NeoRequest<NeoGetBlock, NeoBlock> {
        let full_tx = if full_tx { 1 } else { 0 };
        NeoRequest::new("getblock",
            vec![index.to_value(), full_tx.to_value()],

        )
    }

    fn get_raw_block_by_index(&self, index: i32) -> NeoRequest<NeoGetRawBlock, String> {
        NeoRequest::new("getblock",
            vec![index.to_value(), 0])
    }

    fn invoke_function_diagnostics(&self, contract_hash: H160, name: String, params: Vec<ContractParameter>, signers: Vec<T>) -> NeoRequest<NeoInvokeFunction, InvocationResult> {
        let params = vec![
            contract_hash.to_value(),
            name.to_value(),
            serde_json::to_string(&params).unwrap().to_value(),
            serde_json::to_string(&signers).unwrap().to_value(),
            true.to_value()
        ];

        NeoRequest::new("invokefunction",
            params,

        )
    }

    fn invoke_script_diagnostics(&self, hex: String, signers: Vec<T>) -> NeoRequest<NeoInvokeScript, InvocationResult> {
        let params = vec![hex.to_value(), signers.to_value(), true.to_value()];

        NeoRequest::new("invokescript",
            params,

        )
    }

    fn traverse_iterator(&self, session_id: String, iterator_id: String, count: i32) -> NeoRequest<NeoTraverseIterator, Vec<StackItem>> {
        let params = vec![session_id.to_value(), iterator_id.to_value(), count.to_value()];

        NeoRequest::new("traverseiterator",
            params,
        )
    }

    async fn terminate_session(&self, session_id: &String) -> NeoRequest<NeoTerminateSession,bool> {
        NeoRequest::new("terminatesession", vec![session_id.to_value()])
    }

    async fn invoke_contract_verify(&self, hash: H160, params: Vec<ContractParameter>, signers: Vec<T>) -> NeoRequest<NeoInvokeContractVerify,InvocationResult> {
        let signers = signers
            .into_iter()
            .map(TransactionSigner::from)
            .collect();

        NeoRequest::new("invokecontractverify", vec![hash.to_value(), params, signers])
    }

    async fn get_raw_mempool(&self) -> NeoRequest<NeoGetRawMemPool,MemPoolDetails> {
        NeoRequest::new("getrawmempool", vec![])
    }

    async fn import_private_key(&self, wif: String) -> NeoRequest<NeoImportPrivKey,NeoAddress> {
        NeoRequest::new("importprivkey", vec![wif.to_value()])
   }

    async fn get_block_header_hash(&self, hash: H256) -> NeoRequest<NeoGetBlock,NeoBlock> {
        NeoRequest::new("getblockheader", vec![hash.to_value(), 1])
    }

    async fn send_to_address_send_token(&self, send_token: &TransactionSendToken) -> NeoRequest<NeoSendToAddress, Transaction> {
        let params = [send_token.to_value()].to_vec();
         NeoRequest::new("sendtoaddress", params)
    }

    async fn send_from_send_token(&self, send_token: &TransactionSendToken, from: H160) -> NeoRequest<TransactionSendToken,Transaction> {
        let params = [from.to_value(), vec![send_token]].to_vec();
         NeoRequest::new("sendmany", params)
    }
}