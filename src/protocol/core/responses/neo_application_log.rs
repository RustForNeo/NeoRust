use primitive_types::H256;
use serde::{Serialize, Deserialize};
use crate::protocol::core::responses::invocation_result::Notification;
use crate::protocol::core::stack_item::StackItem;
use crate::types::vm_state::VMState;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct NeoApplicationLog {
    #[serde(rename = "txid")]
    pub transaction_id: H256,
    pub executions: Vec<Execution>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Execution {
    pub trigger: String,
    #[serde(rename = "vmstate")]
    pub state: VMState,
    pub exception: Option<String>,
    #[serde(rename = "gasconsumed")]
    pub gas_consumed: String,
    pub stack: Vec<StackItem>,
    pub notifications: Vec<Notification>,
}