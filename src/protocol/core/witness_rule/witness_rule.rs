use crate::protocol::core::witness_rule::{
	witness_action::WitnessAction, witness_condition::WitnessCondition,
};
use futures::TryStreamExt;
use serde::{ser::SerializeTuple, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessRule {
	pub action: WitnessAction,
	pub condition: WitnessCondition,
}

impl WitnessRule {
	pub fn new(action: WitnessAction, condition: WitnessCondition) -> Self {
		Self { action, condition }
	}
}
