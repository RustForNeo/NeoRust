use crate::{
	neo_error::NeoError,
	protocol::core::responses::{
		contract_state::ContractState, contract_storage_entry::ContractStorageEntry,
		express_shutdown::ExpressShutdown, nep17contract::Nep17Contract,
		oracle_request::OracleRequest, oracle_response_code::OracleResponseCode,
		populated_blocks::PopulatedBlocks, transaction_attribute::TransactionAttribute,
	},
};
use primitive_types::H160;

pub trait NeoExpress {
	fn get_populated_blocks(&self) -> Result<PopulatedBlocks, NeoError>;

	fn get_nep17_contracts(&self) -> Result<Vec<Nep17Contract>, NeoError>;

	fn get_contract_storage(&self, contract: H160) -> Result<Vec<ContractStorageEntry>, NeoError>;

	fn list_contracts(&self) -> Result<Vec<ContractState>, NeoError>;

	fn create_checkpoint(&self, filename: &str) -> Result<(), NeoError>;

	fn list_oracle_requests(&self) -> Result<Vec<OracleRequest>, NeoError>;

	fn create_oracle_response(
		&self,
		response: TransactionAttribute,
	) -> Result<OracleResponseCode, NeoError>;

	fn shutdown(&self) -> Result<ExpressShutdown, NeoError>;
}
