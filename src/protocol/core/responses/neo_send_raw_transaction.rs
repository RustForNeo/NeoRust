use primitive_types::H256;

#[derive(Debug, Hash, PartialEq, Eq,serde::Serialize, serde::Deserialize, Clone)]
pub struct NeoSendRawTransaction {
    pub send_raw_transaction: Option<RawTransaction>,
}


#[derive(Debug, Hash, PartialEq, Eq,serde::Serialize, serde::Deserialize, Clone)]
    pub struct RawTransaction {
        pub hash: H256
    }


