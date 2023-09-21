use crate::protocol::core::witness_rule::{
	witness_action::WitnessAction, witness_condition::WitnessCondition,
};
use futures::TryStreamExt;
use serde::{ser::SerializeTuple, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug)]
pub struct WitnessRule {
	pub action: WitnessAction,
	pub condition: WitnessCondition,
}

impl WitnessRule {
	pub fn new(action: WitnessAction, condition: WitnessCondition) -> Self {
		Self { action, condition }
	}
}

impl Serialize for WitnessRule {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut tuple = serializer.serialize_tuple(2).unwrap();
		tuple.serialize_element(&self.action.byte()).unwrap();
		tuple.serialize_element(&self.condition.serialize(serializer).unwrap()).unwrap();
		tuple.end()
	}
}

// Deserialization

impl<'de> Deserialize<'de> for WitnessRule {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let (action_byte, condition) = Deserialize::deserialize(deserializer).unwrap();

		let action = WitnessAction::from_u8(action_byte).map_err(serde::de::Error::custom).unwrap();

		Ok(Self { action, condition })
	}
}
