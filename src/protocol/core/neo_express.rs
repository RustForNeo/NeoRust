use neo_rust::types::H160;

pub trait NeoExpress {

    fn get_populated_blocks(&self) -> Result<Vec<PopulatedBlock>, Error>;

    fn get_nep17_contracts(&self) -> Result<Vec<Nep17Contract>, Error>;

    fn get_contract_storage(&self, contract: H160) -> Result<Vec<ContractStorage>, Error>;

    fn list_contracts(&self) -> Result<Vec<ContractState>, Error>;

    fn create_checkpoint(&self, filename: &str) -> Result<(), Error>;

    fn list_oracle_requests(&self) -> Result<Vec<OracleRequest>, Error>;

    fn create_oracle_response(&self, response: OracleResponse) -> Result<(), Error>;

    fn shutdown(&self) -> Result<(), Error>;

}