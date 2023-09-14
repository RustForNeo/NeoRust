use serde::{Serialize, Deserialize};
use std::hash::{Hash, Hasher};
use crate::protocol::core::stack_item::StackItem;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Notification {
    pub contract: H160,
    #[serde(rename = "eventname")]
    pub event_name: String,
    pub state: StackItem,
}

impl Notification {

    pub fn new(contract: H160, event_name: String, state: StackItem) -> Self {
        Self {
            contract,
            event_name,
            state,
        }
    }

}

impl Hash for Notification {

    fn hash<H: Hasher>(&self, state: &mut H) {
        self.contract.hash(state);
        self.event_name.hash(state);
        self.state.hash(state);
    }

}