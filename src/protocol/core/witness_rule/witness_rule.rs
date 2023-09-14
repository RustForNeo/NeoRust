use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeTuple;
use crate::protocol::core::witness_rule::witness_action::WitnessAction;
use crate::protocol::core::witness_rule::witness_condition::WitnessCondition;
use crate::utils::r#enum::ByteEnum;

#[derive(Clone, PartialEq, Debug)]
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
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&self.action.byte())?;
        tuple.serialize_element(&self.condition.serialize(serializer)?)?;
        tuple.end()
    }
}

// Deserialization

impl<'de> Deserialize<'de> for WitnessRule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let (action_byte, condition) = Deserialize::deserialize(deserializer)?;

        let action = WitnessAction::from_byte(action_byte)
            .map_err(serde::de::Error::custom)?;

        Ok(Self { action, condition })
    }
}