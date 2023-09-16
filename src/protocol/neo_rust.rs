use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use futures::{SinkExt, Stream, stream};
use lazy_static::lazy_static;
use primitive_types::{H160, H256};
use reqwest::Url;
use serde_json::Value;
use crate::protocol::core::neo_trait::Neo;
use crate::protocol::core::request::Request;
use crate::protocol::core::responses::contract_state::ContractState;
use crate::protocol::core::responses::invocation_result::InvocationResult;
use crate::protocol::core::responses::native_contract_state::NativeContractState;
use crate::protocol::core::responses::neo_address::NeoAddress;
use crate::protocol::core::responses::neo_application_log::NeoApplicationLog;
use crate::protocol::core::responses::neo_block::NeoBlock;
use crate::protocol::core::responses::neo_find_states::States;
use crate::protocol::core::responses::neo_get_nep11balances::Nep11Balances;
use crate::protocol::core::responses::neo_get_nep11transfers::Nep11Transfers;
use crate::protocol::core::responses::neo_get_nep17balances::Nep17Balances;
use crate::protocol::core::responses::neo_get_nep17transfers::Nep17Transfers;
use crate::protocol::core::responses::neo_get_next_block_validators::Validator;
use crate::protocol::core::responses::neo_get_peers::Peers;
use crate::protocol::core::responses::neo_get_state_height::StateHeight;
use crate::protocol::core::responses::neo_get_state_root::StateRoot;
use crate::protocol::core::responses::neo_get_token_balances::TokenBalance;
use crate::protocol::core::responses::neo_get_unclaimed_gas::GetUnclaimedGas;
use crate::protocol::core::responses::neo_get_version::NeoVersion;
use crate::protocol::core::responses::neo_list_plugins::{NeoListPlugins, Plugin};
use crate::protocol::core::responses::neo_network_fee::NeoNetworkFee;
use crate::protocol::core::responses::neo_response_aliases::{NeoGetRawMemPool, NeoTerminateSession};
use crate::protocol::core::responses::neo_send_raw_transaction::RawTransaction;
use crate::protocol::core::responses::neo_validate_address::NeoValidateAddress;
use crate::protocol::core::responses::transaction::Transaction;
use crate::protocol::core::responses::transaction_send_token::TransactionSendToken;
use crate::protocol::core::responses::transaction_signer::TransactionSigner;
use crate::protocol::http_service::HttpService;
use crate::protocol::neo_config::NeoConfig;
use crate::protocol::neo_service::NeoService;
use crate::protocol::protocol_error::ProtocolError;
use crate::transaction::signer::Signer;
use crate::types::{Address, Bytes, H160Externsion};
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

    pub fn subscribe_to_new_blocks(
        &self,
        full_tx: bool,
    ) -> impl Stream<Item = Result<NeoBlock, ProtocolError>> {
        let interval = Duration::from_secs(self.config.polling_interval);
        let mut rx = self.neo_service.block_publisher(full_tx, interval);

        futures::stream::unfold(rx, |mut rx| async {
            rx.next().await.transpose()
        })
    }

    fn get_neo_service(&self) -> &dyn NeoService {
        &self.neo_service.lock().unwrap()
    }

    fn get_neo_service_mut(&mut self) -> &mut dyn NeoService {
        &mut self.neo_service
    }

    async fn dump_private_key(&self, script_hash: H160) -> Result<String, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("dumpprivkey", vec![Value::String(script_hash.to_address())]))
            .await
    }

    pub async fn block_publisher(&self, full_tx: bool) -> Result<impl Stream<Item = Result<NeoBlock, ProtocolError>>, ProtocolError> {
        let interval = Duration::from_millis(self.config.polling_interval);

        let stream = stream::unfold((), move |_| async {
            let block_hash = self.get_best_block_hash().send().await?.result;
            let block = self.get_block(block_hash, full_tx).await?;

            Some((block, ()))
        })
            .throttle(interval);

        Ok(stream)
    }
}

impl Neo for NeoRust {

    async fn get_network_magic_number_bytes(&mut self) -> Result<Bytes, ProtocolError> {
        let magic_int = self.get_network_magic_number().await? & 0xFFFF_FFFF;
        Ok(magic_int.to_be_bytes().to_vec())
    }

    async fn get_network_magic_number(&mut self) -> Result<i32, ProtocolError> {
        if self.config.network_magic.is_none() {
            let magic = self.get_version()
                .await?
                .get_result()
                .protocol
                .ok_or(ProtocolError::IllegalState("Unable to read Network Magic Number from Version".to_string()))?
                .network;
            self.config.set_network_magic(magic).expect("Unable to set Network Magic Number");
        }
        Ok(self.config.network_magic.unwrap() as i32)
    }

    // Blockchain methods

    async fn get_best_block_hash(&self) -> Result<H256, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getbestblockhash", vec![]))
            .await
    }

    async fn get_block_hash(&self, block_index: i32) -> Result<H256, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getblockhash", [Value::from(block_index)].to_vec()))
            .await
    }

    async fn get_block(&self, block_hash: H256, full_tx: bool) -> Result<NeoBlock, ProtocolError> {
        if full_tx {
            self.get_neo_service()
                .send(Request::new("getblock", [Value::from(block_hash), Value::from(1)].to_vec()))
                .await
        } else {
            self.get_block_header(block_hash).await
        }
    }

    // More methods...

    async fn get_nep17_balances(&self, script_hash: H160) -> Result<Nep17Balances, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getnep17balances", [Value::from(script_hash.to_address())].to_vec()))
            .await
    }

    // Node methods

    async fn get_connection_count(&self) -> Result<i32, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getconnectioncount", vec![]))
            .await
    }

    async fn get_peers(&self) -> Result<Peers, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getpeers", vec![]))
            .await
    }

    async fn get_version(&self) -> Result<NeoVersion, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getversion", vec![]))
            .await
    }

    async fn send_raw_transaction(&self, hex: String) -> Result<RawTransaction, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("sendrawtransaction", &[Value::from(hex)]))
            .await
    }

// Smart contract methods

    async fn invoke_function(
        &self,
        contract_hash: &H160,
        method: String,
        params: Vec<ContractParameter>,
        signers: Vec<dyn Signer>,
    ) -> Result<InvocationResult, ProtocolError> {
        let signers = signers
            .into_iter()
            .map(TransactionSigner::from)
            .collect();

        self.get_neo_service()
            .send(Request::new("invokefunction", vec![
                Value::String(contract_hash.to_string()),
                method,
                params,
                signers,
            ]))
            .await
    }

    async fn invoke_script(&self, hex: String, signers: Vec<dyn Signer>) -> Result<InvocationResult, ProtocolError> {
        let signers = signers
            .into_iter()
            .map(TransactionSigner::from)
            .collect();

        self.get_neo_service()
            .send(Request::new("invokescript", vec![hex, signers]))
            .await
    }

// Utility methods

    async fn validate_address(&self, address: String) -> Result<NeoValidateAddress, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("validateaddress", vec![Value::String(address)]))
            .await
    }

// Wallet methods

    async fn close_wallet(&self) -> Result<bool, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("closewallet", vec![]))
            .await
    }



    async fn get_wallet_balance(&self, token_hash: H160) -> Result<dyn TokenBalance, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getwalletbalance", &[token_hash.to_string()]))
            .await
    }

    async fn get_new_address(&self) -> Result<String, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getnewaddress", vec![]))
            .await
    }

     async fn import_private_key(&self, wif: String) -> Result<NeoAddress, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("importprivkey", &[wif]))
            .await
    }

// Application logs

     async fn get_application_log(&self, tx_hash: H256) -> Result<NeoApplicationLog, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getapplicationlog", &[tx_hash.to_string()]))
            .await
    }

// State service

     async fn get_proof(&self, root_hash: H256, contract_hash: H160, key: String) -> Result<String, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getproof", &[
                root_hash.to_string(),
                contract_hash.to_string(),
                key.as_bytes().to_base64(Config::STANDARD),
            ]))
            .await
    }

     async fn get_state(&self, root_hash: H256, contract_hash: H160, key: String) -> Result<String, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getstate", &[
                root_hash.to_string(),
                contract_hash.to_string(),
                key.as_bytes().to_base64( Config::STANDARD),
            ]))
            .await
    }
// Blockchain methods

     async fn get_raw_block(&self, block_hash: H256) -> Result<String, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getblock", &[block_hash.to_string(), 0]))
            .await
    }

    async fn get_block_count(&self) -> Result<i32, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getblockcount", vec![]))
            .await
    }

     async fn get_block_header(&self, index: i32) -> Result<NeoBlock, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getblockheader", &[index, 1]))
            .await
    }

     async fn get_transaction(&self, hash: H256) -> Result<Transaction, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getrawtransaction", &[hash.to_string(), 1]))
            .await
    }

// Smart contract methods

     async fn get_contract_state(&self, hash: H160) -> Result<ContractState, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getcontractstate", &[hash.to_string()]))
            .await
    }

    async fn invoke_contract_verify(&self, hash: H160, params: Vec<ContractParameter>, signers: Vec<dyn Signer>) -> Result<InvocationResult, ProtocolError> {
        let signers = signers
            .into_iter()
            .map(TransactionSigner::from)
            .collect();

        self.get_neo_service()
            .send(Request::new("invokecontractverify", &[hash.to_string(), params, signers]))
            .await
    }
// More node methods

    async fn submit_block(&self, hex: String) -> Result<bool, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("submitblock", &[hex]))
            .await
    }

// More blockchain methods

    async fn get_raw_mempool(&self) -> Result<NeoGetRawMemPool, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getrawmempool", vec![]))
            .await
    }

    async fn get_committee(&self) -> Result<Vec<String>, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getcommittee", vec![]))
            .await
    }

// More smart contract methods

    async fn get_unclaimed_gas(&self, hash: H160) -> Result<GetUnclaimedGas, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("getunclaimedgas", &[hash.as_bytes().scripthash_to_address()]))
            .await
    }

    async fn terminate_session(&self, session_id: &String) -> Result<NeoTerminateSession, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("terminatesession", &[session_id]))
            .await
    }

// More utility methods

    async fn list_plugins(&self) -> Result<NeoListPlugins, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("listplugins", vec![]))
            .await
    }

// More wallet methods

    async fn open_wallet(&self, path: String, password: String) -> Result<bool, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("openwallet", &[path, password]))
            .await
    }

    async fn calculate_network_fee(&self, hex: String) -> Result<NeoNetworkFee, ProtocolError> {
        self.get_neo_service()
            .send(Request::new("calculatenetworkfee", vec![Value::String(hex)]))
            .await
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

    async fn get_block_header_count(&self) -> Result<u32, Box<dyn Error>> {
        let req = Request::new("getblockheadercount", vec![]);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn get_native_contracts(&self) -> Result<Vec<NativeContractState>, Box<dyn Error>> {
        let req = Request::new("getnativecontracts", vec![]);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn get_storage(&self, contract_hash: H160, key: String) -> Result<String, Box<dyn Error>> {
        let params = [contract_hash, key];
        let req = Request::new("getstorage", &params);
        self.get_neo_service()
            .send(req)
            .await
    }

    async fn get_transaction_height(&self, tx_hash: H256) -> Result<u32, Box<dyn Error>> {
        let params = [tx_hash];
        let req = Request::new("gettransactionheight", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn get_next_block_validators(&self) -> Result<Vec<Validator>, Box<dyn Error>> {
        let req = Request::new("getnextblockvalidators", vec![]);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn dump_priv_key(&self, script_hash: H160) -> Result<String, Box<dyn Error>> {
        let params = [script_hash];
        let req = Request::new("dumpprivkey", &params);
                self.get_neo_service()
            .send(req)
            .await
    }
    async fn import_priv_key(&self, priv_key: String) -> Result<Address, ProtocolError> {
        let params = [priv_key];
        let req = Request::new("importprivkey", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn list_address(&self) -> Result<Vec<Address>, ProtocolError> {
        let req = Request::new("listaddress", vec![]);
                self.get_neo_service()
            .send(req)
            .await
    }

// Transaction methods

    async fn send_from(&self, token_hash: H160, from: H160, to: H160, amount: u32) -> Result<Transaction, ProtocolError> {
        let params = [token_hash, from, to, amount];
        let req = Request::new("sendfrom", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn send_from_send_token(&self, send_token: &TransactionSendToken, from: H160) -> Result<Transaction, ProtocolError> {
        let params = [from, vec![send_token]];
        let req = Request::new("sendmany", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn send_many(&self, from: Option<H160>, send_tokens: Vec<TransactionSendToken>) -> Result<Transaction, ProtocolError> {
        let params = [from, send_tokens];
        let req = Request::new("sendmany", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn send_to_address(&self, token_hash: H160, to: H160, amount: u32) -> Result<Transaction, ProtocolError> {
        let params = [token_hash, to, amount];
        let req = Request::new("sendtoaddress", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn send_to_address_send_token(&self, send_token: &TransactionSendToken) -> Result<Transaction, ProtocolError> {
        let params = [send_token];
        let req = Request::new("sendtoaddress", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

// NEP-17 methods

    async fn get_nep17_transfers(&self, script_hash: H160) -> Result<Nep17Transfers, ProtocolError> {
        let params = [script_hash];
        let req = Request::new("getnep17transfers", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn get_nep17_transfers_from(&self, script_hash: H160, from: i64) -> Result<Nep17Transfers, ProtocolError> {
        let params = [script_hash, from];
        let req = Request::new("getnep17transfers", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn get_nep17_transfers_range(&self, script_hash: H160, from: i64, to: i64) -> Result<Nep17Transfers, ProtocolError> {
        let params = [script_hash, from, to];
        let req = Request::new("getnep17transfers", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

// NEP-11 methods

    async fn get_nep11_balances(&self, script_hash: H160) -> Result<Nep11Balances, ProtocolError> {
        let params = [script_hash];
        let req = Request::new("getnep11balances", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn get_nep11_transfers(&self, script_hash: H160) -> Result<Nep11Transfers, ProtocolError> {
        let params = [script_hash];
        let req = Request::new("getnep11transfers", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn get_nep11_transfers_from(&self, script_hash: H160, from: i64) -> Result<Nep11Transfers, ProtocolError> {
        let params = [script_hash, from];
        let req = Request::new("getnep11transfers", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn get_nep11_transfers_range(&self, script_hash: H160, from: i64, to: i64) -> Result<Nep11Transfers, ProtocolError> {
        let params = [script_hash, from, to];
        let req = Request::new("getnep11transfers", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn get_nep11_properties(&self, script_hash: H160, token_id: String) -> Result<HashMap<String, String>, ProtocolError> {
        let params = [script_hash, token_id];
        let req = Request::new("getnep11properties", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

// State service methods

    async fn get_state_root(&self, block_index: u32) -> Result<StateRoot, ProtocolError> {
        let params = [block_index];
        let req = Request::new("getstateroot", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn verify_proof(&self, root_hash: H256, proof: String) -> Result<bool, ProtocolError> {
        let params = [root_hash, proof];
        let req = Request::new("verifyproof", &params);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn get_state_height(&self) -> Result<StateHeight, ProtocolError> {
        let req = Request::new("getstateheight", vec![]);
                self.get_neo_service()
            .send(req)
            .await
    }

    async fn find_states(&self, root_hash: H256, contract_hash: H160, key_prefix: String, start_key: Option<String>, count: Option<u32>) -> Result<States, ProtocolError> {
        let mut params = vec![root_hash, contract_hash, key_prefix];
        if let Some(start_key) = start_key {
            params.push(start_key)
        }
        if let Some(count) = count {
            params.push(count)
        }

        let req = Request::new("findstates", &params);
                self.get_neo_service()
            .send(req)
            .await
    }
}