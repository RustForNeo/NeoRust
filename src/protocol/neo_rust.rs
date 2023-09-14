use std::sync::MutexGuard;
use std::time::Duration;
use futures::{SinkExt, Stream};
use lazy_static::lazy_static;
use crate::protocol::core::responses::contract_state::ContractState;
use crate::protocol::core::responses::invocation_result::InvocationResult;
use crate::protocol::core::responses::neo_address::NeoAddress;
use crate::protocol::core::responses::neo_application_log::NeoApplicationLog;
use crate::protocol::core::responses::neo_block::NeoBlock;
use crate::protocol::core::responses::neo_get_nep17balances::Nep17Balances;
use crate::protocol::core::responses::neo_get_peers::Peers;
use crate::protocol::core::responses::neo_get_token_balances::TokenBalance;
use crate::protocol::core::responses::neo_get_version::NeoVersion;
use crate::protocol::core::responses::neo_list_plugins::Plugin;
use crate::protocol::core::responses::neo_network_fee::NeoNetworkFee;
use crate::protocol::core::responses::neo_send_raw_transaction::RawTransaction;
use crate::protocol::core::responses::neo_validate_address::NeoValidateAddress;
use crate::protocol::core::responses::transaction::Transaction;
use crate::protocol::core::responses::transaction_signer::TransactionSigner;
use crate::protocol::neo_config::NeoConfig;
use crate::protocol::neo_service::NeoService;
use crate::protocol::protocol_error::ProtocolError;
use crate::transaction::signer::Signer;
use crate::types::Bytes;
use crate::types::contract_parameter::ContractParameter;

lazy_static! {
  pub static ref NEO_RUST_INSTANCE: Mutex<NeoRust> =
       Mutex::new(NeoRust::new());
}

pub struct NeoRust {
    config: NeoConfig,
    neo_service: dyn NeoService,
}

impl NeoRust {

    pub fn instance() -> MutexGuard<'static, Self> {
        NEO_RUST_INSTANCE.lock().unwrap()
    }
    pub fn build(neo_service: Box<dyn NeoService>, config: NeoConfig) -> Self {
        Self {
            config,
            neo_service,
        }
    }

    pub fn get_config(&self) -> &NeoConfig {
        &self.config
    }
    pub fn get_config_mut(&mut self) -> &mut NeoConfig {
        &mut self.config
    }

    pub fn get_neo_service(&self) -> &dyn NeoService {
        &self.neo_service
    }
    pub fn get_neo_service_mut(&mut self) -> &mut dyn NeoService {
        &mut self.neo_service
    }
    pub async fn get_network_magic_number_bytes(&mut self) -> Result<Bytes, ProtocolError> {
        let magic_int = self.get_network_magic_number().await? & 0xFFFF_FFFF;
        Ok(magic_int.to_be_bytes())
    }

    pub async fn get_network_magic_number(&mut self) -> Result<i32, ProtocolError> {
        if self.config.network_magic.is_none() {
            let magic = self.get_version()
                .send(())
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

    pub async fn get_best_block_hash(&self) -> Result<H256, ProtocolError> {
        self.neo_service
            .send_request("getbestblockhash", &[])
            .await
    }

    pub async fn get_block_hash(&self, block_index: i32) -> Result<H256, ProtocolError> {
        self.neo_service
            .send_request("getblockhash", &[block_index])
            .await
    }

    pub async fn get_block(&self, block_hash: H256, full_tx: bool) -> Result<NeoBlock, ProtocolError> {
        if full_tx {
            self.neo_service
                .send_request("getblock", &[block_hash.to_string(), 1])
                .await
        } else {
            self.get_block_header(block_hash).await
        }
    }

    // More methods...

    pub async fn get_nep17_balances(&self, script_hash: H160) -> Result<Nep17Balances, ProtocolError> {
        self.neo_service
            .send_request("getnep17balances", &[script_hash.to_address()])
            .await
    }

    // Subscription methods

    pub fn subscribe_to_new_blocks(
        &self,
        full_tx: bool,
    ) -> impl Stream<Item = Result<NeoBlock, ProtocolError>> {
        let interval = Duration::from_secs(&self.config.polling_interval);
        let mut rx = self.neo_service.block_publisher(full_tx, interval);

        futures::stream::unfold(rx, |mut rx| async {
            rx.next().await.transpose()
        })
    }

    // Node methods

    pub async fn get_connection_count(&self) -> Result<i32, ProtocolError> {
        self.neo_service
            .send_request("getconnectioncount", &[])
            .await
    }

    pub async fn get_peers(&self) -> Result<Peers, ProtocolError> {
        self.neo_service
            .send_request("getpeers", &[])
            .await
    }

    pub async fn get_version(&self) -> Result<NeoVersion, ProtocolError> {
        self.neo_service
            .send_request("getversion", &[])
            .await
    }

    pub async fn send_raw_transaction(&self, hex: String) -> Result<RawTransaction, ProtocolError> {
        self.neo_service
            .send_request("sendrawtransaction", &[hex])
            .await
    }

// Smart contract methods

    pub async fn invoke_function(
        &self,
        contract_hash: &H160,
        method: String,
        params: Vec<ContractParameter>,
        signers: Vec<Signer>,
    ) -> Result<InvocationResult, ProtocolError> {
        let signers = signers
            .into_iter()
            .map(TransactionSigner::from)
            .collect();

        self.neo_service
            .send_request("invokefunction", &[
                contract_hash.to_string(),
                method,
                params,
                signers,
            ])
            .await
    }

    pub async fn invoke_script(&self, hex: String, signers: Vec<Signer>) -> Result<InvocationResult, ProtocolError> {
        let signers = signers
            .into_iter()
            .map(TransactionSigner::from)
            .collect();

        self.neo_service
            .send_request("invokescript", &[hex, signers])
            .await
    }

// Utility methods

    pub async fn validate_address(&self, address: String) -> Result<NeoValidateAddress, ProtocolError> {
        self.neo_service
            .send_request("validateaddress", &[address])
            .await
    }

// Wallet methods

    pub async fn close_wallet(&self) -> Result<bool, ProtocolError> {
        self.neo_service
            .send_request("closewallet", &[])
            .await
    }

    pub async fn dump_private_key(&self, script_hash: H160) -> Result<String, ProtocolError> {
        self.neo_service
            .send_request("dumpprivkey", &[script_hash.to_address()])
            .await
    }

    pub async fn get_wallet_balance(&self, token_hash: H160) -> Result<dyn TokenBalance, ProtocolError> {
        self.neo_service
            .send_request("getwalletbalance", &[token_hash.to_string()])
            .await
    }

    pub async fn get_new_address(&self) -> Result<String, ProtocolError> {
        self.neo_service
            .send_request("getnewaddress", &[])
            .await
    }

    pub async fn import_private_key(&self, wif: String) -> Result<NeoAddress, ProtocolError> {
        self.neo_service
            .send_request("importprivkey", &[wif])
            .await
    }

// Application logs

    pub async fn get_application_log(&self, tx_hash: H256) -> Result<NeoApplicationLog, ProtocolError> {
        self.neo_service
            .send_request("getapplicationlog", &[tx_hash.to_string()])
            .await
    }

// State service

    pub async fn get_proof(&self, root_hash: H256, contract_hash: H160, key: String) -> Result<String, ProtocolError> {
        self.neo_service
            .send_request("getproof", &[
                root_hash.to_string(),
                contract_hash.to_string(),
                key.as_bytes().to_base64(Config::STANDARD),
            ])
            .await
    }

    pub async fn get_state(&self, root_hash: H256, contract_hash: H160, key: String) -> Result<String, ProtocolError> {
        self.neo_service
            .send_request("getstate", &[
                root_hash.to_string(),
                contract_hash.to_string(),
                key.as_bytes().to_base64(Config {}),
            ])
            .await
    }
// Blockchain methods

    pub async fn get_raw_block(&self, block_hash: H256) -> Result<String, ProtocolError> {
        self.neo_service
            .send_request("getblock", &[block_hash.to_string(), 0])
            .await
    }

    pub async fn get_block_count(&self) -> Result<i32, ProtocolError> {
        self.neo_service
            .send_request("getblockcount", &[])
            .await
    }

    pub async fn get_block_header(&self, index: i32) -> Result<NeoBlock, ProtocolError> {
        self.neo_service
            .send_request("getblockheader", &[index, 1])
            .await
    }

    pub async fn get_transaction(&self, hash: H256) -> Result<Transaction, ProtocolError> {
        self.neo_service
            .send_request("getrawtransaction", &[hash.to_string(), 1])
            .await
    }

// Smart contract methods

    pub async fn get_contract_state(&self, hash: H160) -> Result<ContractState, ProtocolError> {
        self.neo_service
            .send_request("getcontractstate", &[hash.to_string()])
            .await
    }

    pub async fn invoke_contract_verify(&self, hash: H160, params: Vec<ContractParameter>, signers: Vec<Signer>) -> Result<sc::InvocationResult, ProtocolError> {
        let signers = signers
            .into_iter()
            .map(TransactionSigner::from)
            .collect();

        self.neo_service
            .send_request("invokecontractverify", &[hash.to_string(), params, signers])
            .await
    }
// More node methods

    pub async fn submit_block(&self, hex: String) -> Result<bool, ProtocolError> {
        self.neo_service
            .send_request("submitblock", &[hex])
            .await
    }

// More blockchain methods

    pub async fn get_raw_mempool(&self) -> Result<Vec<H256>, ProtocolError> {
        self.neo_service
            .send_request("getrawmempool", &[])
            .await
    }

    pub async fn get_committee(&self) -> Result<Vec<String>, ProtocolError> {
        self.neo_service
            .send_request("getcommittee", &[])
            .await
    }

// More smart contract methods

    pub async fn get_unclaimed_gas(&self, hash: H160) -> Result<UnclaimedGas, ProtocolError> {
        self.neo_service
            .send_request("getunclaimedgas", &[hash.to_address()])
            .await
    }

    pub async fn terminate_session(&self, session_id: String) -> Result<bool, ProtocolError> {
        self.neo_service
            .send_request("terminatesession", &[session_id])
            .await
    }

// More utility methods

    pub async fn list_plugins(&self) -> Result<Vec<Plugin>, ProtocolError> {
        self.neo_service
            .send_request("listplugins", &[])
            .await
    }

// More wallet methods

    pub async fn open_wallet(&self, path: String, password: String) -> Result<bool, ProtocolError> {
        self.neo_service
            .send_request("openwallet", &[path, password])
            .await
    }

    pub async fn calculate_network_fee(&self, hex: String) -> Result<NeoNetworkFee, ProtocolError> {
        self.neo_service
            .send_request("calculatenetworkfee", &[hex])
            .await
    }
}