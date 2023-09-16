use primitive_types::H160;
use serde_json::Value;
use crate::protocol::core::request::Request;
use crate::protocol::core::responses::contract_storage_entry::ContractStorageEntry;
use crate::protocol::core::responses::express_contract_state::ExpressContractState;
use crate::protocol::core::responses::express_shutdown::ExpressShutdown;
use crate::protocol::core::responses::neo_response_aliases::{NeoExpressCreateCheckpoint, NeoExpressCreateOracleResponseTx, NeoExpressGetContractStorage, NeoExpressGetNep17Contracts, NeoExpressGetPopulatedBlocks, NeoExpressListContracts, NeoExpressListOracleRequests, NeoExpressShutdown};
use crate::protocol::core::responses::nep17contract::Nep17Contract;
use crate::protocol::core::responses::oracle_request::OracleRequest;
use crate::protocol::core::responses::populated_blocks::PopulatedBlocks;
use crate::protocol::core::responses::transaction_attribute::TransactionAttribute;
use crate::protocol::neo_service::NeoService;

pub struct NeoRustExpress {
    neo_service: dyn NeoService,
}

impl NeoRustExpress {

    pub fn express_get_populated_blocks(&self) -> Request<NeoExpressGetPopulatedBlocks, PopulatedBlocks> {
        Request::new(
            "expressgetpopulatedblocks",
            vec![],
            
        )
    }

    pub fn express_get_nep17_contracts(&self) -> Request<NeoExpressGetNep17Contracts, Vec<Nep17Contract>> {
        Request::new(
            "expressgetnep17contracts",
            vec![],
            
        )
    }

    pub fn express_get_contract_storage(&self, contract_hash: H160) -> Request<NeoExpressGetContractStorage, Vec<ContractStorageEntry>> {
        Request::new(
            "expressgetcontractstorage",
            vec![Value::String(contract_hash.to_string())],
            
        )
    }

    pub fn express_list_contracts(&self) -> Request<NeoExpressListContracts, Vec<ExpressContractState>> {
        Request::new(
            "expresslistcontracts",
            vec![],
            
        )
    }

    pub fn express_create_checkpoint(&self, filename: String) -> Request<NeoExpressCreateCheckpoint, String> {
        Request::new(
            "expresscreatecheckpoint",
            vec![Value::String(filename)],
            
        )
    }

    pub fn express_list_oracle_requests(&self) -> Request<NeoExpressListOracleRequests, Vec<OracleRequest>> {
        Request::new(
            "expresslistoraclerequests",
            vec![],
            
        )
    }

    pub fn express_create_oracle_response_tx(&self, oracle_response: TransactionAttribute) -> Request<NeoExpressCreateOracleResponseTx, String> {
        Request::new(
            "expresscreateoracleresponsetx",
            vec![oracle_response],
        )
    }

    pub fn express_shutdown(&self) -> Request<NeoExpressShutdown, ExpressShutdown> {
        Request::new(
            "expressshutdown",
            vec![]
        )
    }
}