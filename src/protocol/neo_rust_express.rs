use primitive_types::H160;
use crate::protocol::core::request::Request;
use crate::protocol::core::responses::neo_response_aliases::{NeoExpressGetNep17Contracts, NeoExpressGetPopulatedBlocks};
use crate::protocol::core::responses::nep17contract::Nep17Contract;
use crate::protocol::core::responses::populated_blocks::PopulatedBlocks;
use crate::protocol::neo_service::NeoService;

pub struct NeoRustExpress {
    neo_service: NeoService,
}

impl NeoRustExpress {

    pub fn express_get_populated_blocks(&self) -> Request<NeoExpressGetPopulatedBlocks, PopulatedBlocks> {
        Request::new(
            "expressgetpopulatedblocks".to_owned(),
            vec![],
            self.neo_service.clone(),
        )
    }

    pub fn express_get_nep17_contracts(&self) -> Request<NeoExpressGetNep17Contracts, Vec<Nep17Contract>> {
        Request::new(
            "expressgetnep17contracts".to_owned(),
            vec![],
            self.neo_swift_service.clone(),
        )
    }

    pub fn express_get_contract_storage(&self, contract_hash: H160) -> Request<NeoExpressGetContractStorage, Vec<ContractStorageEntry>> {
        Request::new(
            "expressgetcontractstorage".to_owned(),
            vec![contract_hash.to_string()],
            self.neo_swift_service.clone(),
        )
    }

    pub fn express_list_contracts(&self) -> Request<NeoExpressListContracts, Vec<ExpressContractState>> {
        Request::new(
            "expresslistcontracts".to_owned(),
            vec![],
            self.neo_swift_service.clone(),
        )
    }

    pub fn express_create_checkpoint(&self, filename: String) -> Request<NeoExpressCreateCheckpoint, String> {
        Request::new(
            "expresscreatecheckpoint".to_owned(),
            vec![filename],
            self.neo_swift_service.clone(),
        )
    }

    pub fn express_list_oracle_requests(&self) -> Request<NeoExpressListOracleRequests, Vec<OracleRequest>> {
        Request::new(
            "expresslistoraclerequests".to_owned(),
            vec![],
            self.neo_swift_service.clone(),
        )
    }

    pub fn express_create_oracle_response_tx(&self, oracle_response: TransactionAttribute) -> Request<NeoExpressCreateOracleResponseTx, String> {
        Request::new(
            "expresscreateoracleresponsetx".to_owned(),
            vec![oracle_response],
            self.neo_swift_service.clone(),
        )
    }

    pub fn express_shutdown(&self) -> Request<NeoExpressShutdown, ExpressShutdown> {
        Request::new(
            "expressshutdown".to_owned(),
            vec![],
            self.neo_swift_service.clone(),
        )
    }

}