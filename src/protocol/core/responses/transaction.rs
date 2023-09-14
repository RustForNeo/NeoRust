use serde::{Serialize, Deserialize};
use std::hash::{Hash, Hasher};
use neon::types::{H256, H160, VMState};
use crate::protocol::core::responses::neo_witness::NeoWitness;
use crate::types::hash256::H256;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Transaction {
    #[serde(rename = "hash")]
    pub hash: H256,

    #[serde(rename = "size")]
    pub size: i32,

    #[serde(rename = "version")]
    pub version: i32,

    #[serde(rename = "nonce")]
    pub nonce: i32,

    #[serde(rename = "sender")]
    pub sender: H160,

    #[serde(rename = "sysfee")]
    pub sys_fee: String,

    #[serde(rename = "netfee")]
    pub net_fee: String,

    #[serde(rename = "validuntilblock")]
    pub valid_until_block: i32,

    #[serde(rename = "signers")]
    pub signers: Vec<TransactionSigner>,

    #[serde(rename = "attributes")]
    pub attributes: Vec<TransactionAttribute>,

    #[serde(rename = "script")]
    pub script: String,

    #[serde(rename = "witnesses")]
    pub witnesses: Vec<NeoWitness>,

    #[serde(rename = "blockhash")]
    pub block_hash: Option<H256>,

    #[serde(rename = "confirmations")]
    pub confirmations: Option<i32>,

    #[serde(rename = "blocktime")]
    pub block_time: Option<i32>,

    #[serde(rename = "vmstate")]
    pub vm_state: Option<VMState>,
}

impl Hash for Transaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
        self.size.hash(state);
        self.version.hash(state);
        self.nonce.hash(state);
        self.sender.hash(state);
        self.sys_fee.hash(state);
        self.net_fee.hash(state);
        self.valid_until_block.hash(state);
        self.signers.hash(state);
        self.attributes.hash(state);
        self.script.hash(state);
        self.witnesses.hash(state);
        self.block_hash.hash(state);
        self.confirmations.hash(state);
        self.block_time.hash(state);
        self.vm_state.hash(state);
    }
        // etc
}