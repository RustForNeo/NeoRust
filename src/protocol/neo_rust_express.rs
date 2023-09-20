use crate::{
	protocol::{
		core::{
			request::NeoRequest,
			response::NeoResponse,
			responses::{
				contract_storage_entry::ContractStorageEntry,
				express_contract_state::ExpressContractState,
				express_shutdown::ExpressShutdown,
				neo_response_aliases::{
					NeoExpressCreateCheckpoint, NeoExpressCreateOracleResponseTx,
					NeoExpressGetContractStorage, NeoExpressGetNep17Contracts,
					NeoExpressGetPopulatedBlocks, NeoExpressListContracts,
					NeoExpressListOracleRequests, NeoExpressShutdown,
				},
				nep17contract::Nep17Contract,
				oracle_request::OracleRequest,
				populated_blocks::PopulatedBlocks,
				transaction_attribute::TransactionAttribute,
			},
		},
		neo_service::NeoService,
	},
	types::ValueExtension,
};
use primitive_types::H160;
use serde_json::Value;

pub struct NeoRustExpress {
	neo_service: dyn NeoService,
}

impl NeoRustExpress {
	pub fn express_get_populated_blocks(&self) -> NeoRequest<PopulatedBlocks> {
		NeoRequest::new("expressgetpopulatedblocks", vec![])
	}

	pub fn express_get_nep17_contracts(&self) -> NeoRequest<Vec<Nep17Contract>> {
		NeoRequest::new("expressgetnep17contracts", vec![])
	}

	pub fn express_get_contract_storage(
		&self,
		contract_hash: H160,
	) -> NeoRequest<Vec<ContractStorageEntry>> {
		NeoRequest::new("expressgetcontractstorage", vec![Value::String(contract_hash.to_string())])
	}

	pub fn express_list_contracts(&self) -> NeoRequest<Vec<ExpressContractState>> {
		NeoRequest::new("expresslistcontracts", vec![])
	}

	pub fn express_create_checkpoint(&self, filename: String) -> NeoRequest<String> {
		NeoRequest::new("expresscreatecheckpoint", vec![filename.to_value()])
	}

	pub fn express_list_oracle_requests(&self) -> NeoRequest<Vec<OracleRequest>> {
		NeoRequest::new("expresslistoraclerequests", vec![])
	}

	pub fn express_create_oracle_response_tx(
		&self,
		oracle_response: TransactionAttribute,
	) -> NeoRequest<String> {
		NeoRequest::new("expresscreateoracleresponsetx", vec![oracle_response.to_value()])
	}

	pub fn express_shutdown(&self) -> NeoRequest<ExpressShutdown> {
		NeoRequest::new("expressshutdown", vec![])
	}
}
