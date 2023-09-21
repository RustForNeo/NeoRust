use crate::{
	protocol::core::{
		responses::{
			contract_state::ContractState, contract_storage_entry::ContractStorageEntry,
			express_contract_state::ExpressContractState, express_shutdown::ExpressShutdown,
			invocation_result::InvocationResult, native_contract_state::NativeContractState,
			neo_address::NeoAddress, neo_application_log::NeoApplicationLog, neo_block::NeoBlock,
			neo_network_fee::NeoNetworkFee, nep17contract::Nep17Contract,
			oracle_request::OracleRequest, populated_blocks::PopulatedBlocks,
			transaction::Transaction,
		},
		stack_item::StackItem,
	},
	utils::*,
};
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
pub struct NeoBlockCount {
	pub block_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NeoBlockHash {
	#[serde(serialize_with = "serialize_h256")]
	#[serde(deserialize_with = "deserialize_h256")]
	pub block_hash: Option<H256>,
}

pub type NeoBlockHeaderCount = NeoConnectionCount;

#[derive(Debug)]
pub struct NeoCalculateNetworkFee {
	pub network_fee: Option<NeoNetworkFee>,
}

#[derive(Debug)]
pub struct NeoCloseWallet {
	pub close_wallet: Option<bool>,
}

#[derive(Debug)]
pub struct NeoConnectionCount {
	pub count: Option<i32>,
}

#[derive(Debug)]
pub struct NeoDumpPrivKey {
	pub dump_priv_key: Option<String>,
}

#[derive(Debug)]
pub struct NeoExpressCreateCheckpoint {
	pub filename: Option<String>,
}

#[derive(Debug)]
pub struct NeoExpressCreateOracleResponseTx {
	pub oracle_response_tx: Option<String>,
}

#[derive(Debug)]
pub struct NeoExpressGetContractStorage {
	pub contract_storage: Option<Vec<ContractStorageEntry>>,
}

#[derive(Debug)]
pub struct NeoExpressGetNep17Contracts {
	pub nep17_contracts: Option<Vec<Nep17Contract>>,
}

#[derive(Debug)]
pub struct NeoExpressGetPopulatedBlocks {
	pub populated_blocks: Option<PopulatedBlocks>,
}

#[derive(Debug)]
pub struct NeoExpressListContracts {
	pub contracts: Option<Vec<ExpressContractState>>,
}

#[derive(Debug)]
pub struct NeoExpressListOracleRequests {
	pub oracle_requests: Option<Vec<OracleRequest>>,
}

#[derive(Debug)]
pub struct NeoExpressShutdown {
	pub express_shutdown: Option<ExpressShutdown>,
}

#[derive(Debug)]
pub struct NeoGetApplicationLog {
	pub application_log: Option<NeoApplicationLog>,
}

#[derive(Debug)]
pub struct NeoGetBlock {
	pub block: Option<NeoBlock>,
}

#[derive(Debug)]
pub struct NeoGetCommittee {
	pub committee: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct NeoGetContractState {
	pub contract_state: Option<ContractState>,
}

#[derive(Debug)]
pub struct NeoGetNativeContracts {
	pub native_contracts: Option<Vec<NativeContractState>>,
}

#[derive(Debug)]
pub struct NeoGetNep11Properties {
	pub properties: Option<HashMap<String, String>>,
}

#[derive(Debug)]
pub struct NeoGetNewAddress {
	pub address: Option<String>,
}

#[derive(Debug)]
pub struct NeoGetProof {
	pub proof: Option<String>,
}

#[derive(Debug)]
pub struct NeoGetRawBlock {
	pub raw_block: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NeoGetRawMemPool {
	#[serde(serialize_with = "serialize_vec_h256")]
	#[serde(deserialize_with = "deserialize_vec_h256")]
	pub addresses: Option<Vec<H256>>,
}

#[derive(Debug)]
pub struct NeoGetRawTransaction {
	pub raw_transaction: Option<String>,
}

#[derive(Debug)]
pub struct NeoGetState {
	pub state: Option<String>,
}

#[derive(Debug)]
pub struct NeoGetStorage {
	pub storage: Option<String>,
}

#[derive(Debug)]
pub struct NeoGetTransaction {
	pub transaction: Option<Transaction>,
}

#[derive(Debug)]
pub struct NeoGetWalletHeight {
	pub height: Option<i32>,
}

pub type NeoGetTransactionHeight = NeoGetWalletHeight;

#[derive(Debug)]
pub struct NeoGetWalletUnclaimedGas {
	pub wallet_unclaimed_gas: Option<String>,
}

#[derive(Debug)]
pub struct NeoImportPrivKey {
	pub address: Option<NeoAddress>,
}

#[derive(Debug)]
pub struct NeoInvoke {
	pub invocation_result: Option<InvocationResult>,
}

pub type NeoInvokeContractVerify = NeoInvoke;
pub type NeoInvokeFunction = NeoInvoke;
pub type NeoInvokeScript = NeoInvoke;

#[derive(Debug)]
pub struct NeoListAddress {
	pub addresses: Option<Vec<NeoAddress>>,
}

#[derive(Debug)]
pub struct NeoOpenWallet {
	pub open_wallet: Option<bool>,
}

#[derive(Debug)]
pub struct NeoSendFrom {
	pub send_from: Option<Transaction>,
}

#[derive(Debug)]
pub struct NeoSendMany {
	pub send_many: Option<Transaction>,
}

#[derive(Debug)]
pub struct NeoSendToAddress {
	pub send_to_address: Option<Transaction>,
}

#[derive(Debug)]
pub struct NeoSubmitBlock {
	pub submit_block: Option<bool>,
}

#[derive(Debug)]
pub struct NeoTerminateSession {
	pub terminate_session: Option<bool>,
}

#[derive(Debug)]
pub struct NeoTraverseIterator {
	pub traverse_iterator: Option<Vec<StackItem>>,
}

#[derive(Debug)]
pub struct NeoVerifyProof {
	pub verify_proof: Option<String>,
}
