use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct NeoGetVersion {
    pub version: Option<NeoVersion>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct NeoVersion {
    #[serde(rename = "tcpport")]
    pub tcp_port: Option<u16>,
    #[serde(rename = "wsport")]
    pub ws_port: Option<u16>,
    pub nonce: u32,
    #[serde(rename = "useragent")]
    pub user_agent: String,
    pub protocol: NeoProtocol,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct NeoProtocol {
    pub network: u32,
    #[serde(rename = "validatorscount")]
    pub validators_count: Option<u32>,
    #[serde(rename = "msperblock")]
    pub ms_per_block: u32,
    #[serde(rename = "maxvaliduntilblockincrement")]
    pub max_valid_until_block_increment: u32,
    #[serde(rename = "maxtraceableblocks")]
    pub max_traceable_blocks: u32,
    #[serde(rename = "addressversion")]
    pub address_version: u32,
    #[serde(rename = "maxtransactionsperblock")]
    pub max_transactions_per_block: u32,
    #[serde(rename = "memorypoolmaxtransactions")]
    pub memory_pool_max_transactions: u32,
    #[serde(rename = "initialgasdistribution")]
    pub initial_gas_distribution: u64,
}