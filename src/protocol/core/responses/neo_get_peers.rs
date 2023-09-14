use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct NeoGetPeers {
    pub peers: Option<Peers>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Peers {
    pub connected: Vec<AddressEntry>,
    pub bad: Vec<AddressEntry>,
    pub unconnected: Vec<AddressEntry>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct AddressEntry {
    pub address: String,
    pub port: u16,
}