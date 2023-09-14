use primitive_types::H160;
use crate::protocol::core::responses::contract_state::ContractState;
use crate::protocol::core::responses::nep17contract::Nep17Contract;
use crate::protocol::core::responses::oracle_request::OracleRequest;
use crate::protocol::core::responses::populated_blocks::PopulatedBlocks;

pub trait NeoExpress {

    fn get_populated_blocks(&self) -> Result<Vec<PopulatedBlocks>, Error>;

    fn get_nep17_contracts(&self) -> Result<Vec<Nep17Contract>, Error>;

    fn get_contract_storage(&self, contract: H160) -> Result<Vec<ContractStorage>, Error>;

    fn list_contracts(&self) -> Result<Vec<ContractState>, Error>;

    fn create_checkpoint(&self, filename: &str) -> Result<(), Error>;

    fn list_oracle_requests(&self) -> Result<Vec<OracleRequest>, Error>;

    fn create_oracle_response(&self, response: OracleResponse) -> Result<(), Error>;

    fn shutdown(&self) -> Result<(), Error>;

}