use crate::{
	neo_error::NeoError,
	protocol::core::responses::{
		contract_state::ContractState,
		contract_storage_entry::ContractStorageEntry,
		neo_response_aliases::{
			NeoExpressCreateOracleResponseTx, NeoExpressGetNep17Contracts,
			NeoExpressGetPopulatedBlocks, NeoExpressShutdown,
		},
		nep17contract::Nep17Contract,
		oracle_request::OracleRequest,
		populated_blocks::PopulatedBlocks,
		transaction_attribute::TransactionAttribute,
	},
};
use primitive_types::H160;

pub trait NeoExpress {
	fn get_populated_blocks(
		&self,
	) -> Result<(NeoExpressGetPopulatedBlocks, PopulatedBlocks), NeoError>;

	fn get_nep17_contracts(
		&self,
	) -> Result<(NeoExpressGetNep17Contracts, Vec<Nep17Contract>), NeoError>;

	fn get_contract_storage(&self, contract: H160) -> Result<Vec<ContractStorageEntry>, NeoError>;

	fn list_contracts(&self) -> Result<Vec<ContractState>, NeoError>;

	fn create_checkpoint(&self, filename: &str) -> Result<(), NeoError>;

	fn list_oracle_requests(&self) -> Result<Vec<OracleRequest>, NeoError>;

	fn create_oracle_response(
		&self,
		response: TransactionAttribute,
	) -> Result<NeoExpressCreateOracleResponseTx, NeoError>;

	fn shutdown(&self) -> Result<NeoExpressShutdown, NeoError>;
}
