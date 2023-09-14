use primitive_types::H160;
use crate::protocol::core::responses::contract_state::ContractState;
use crate::protocol::core::responses::contract_storage_entry::ContractStorageEntry;
use crate::protocol::core::responses::neo_response_aliases::{NeoExpressCreateOracleResponseTx, NeoExpressGetNep17Contracts, NeoExpressGetPopulatedBlocks, NeoExpressShutdown};
use crate::protocol::core::responses::nep17contract::Nep17Contract;
use crate::protocol::core::responses::oracle_request::OracleRequest;
use crate::protocol::core::responses::populated_blocks::PopulatedBlocks;
use crate::protocol::core::responses::transaction_attribute::TransactionAttribute;

pub trait NeoExpress {

    fn get_populated_blocks(&self) -> Result<(NeoExpressGetPopulatedBlocks, PopulatedBlocks), Err>;

    fn get_nep17_contracts(&self) -> Result<(NeoExpressGetNep17Contracts, Vec<Nep17Contract>), Err>;

    fn get_contract_storage(&self, contract: H160) -> Result<Vec<ContractStorageEntry>, Err>;

    fn list_contracts(&self) -> Result<Vec<ContractState>, Err>;

    fn create_checkpoint(&self, filename: &str) -> Result<(), Err>;

    fn list_oracle_requests(&self) -> Result<Vec<OracleRequest>, Err>;

    fn create_oracle_response(&self, response: TransactionAttribute) -> Result<NeoExpressCreateOracleResponseTx, Err>;

    fn shutdown(&self) -> Result<NeoExpressShutdown, Err>;

}