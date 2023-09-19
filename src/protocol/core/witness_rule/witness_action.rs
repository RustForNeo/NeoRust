use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(
	Display,
	EnumString,
	FromPrimitive,
	Copy,
	Clone,
	Debug,
	PartialEq,
	Eq,
	Hash,
	Serialize,
	Deserialize,
)]
pub enum WitnessAction {
	#[strum(serialize = "Deny")]
	Deny = 0,
	#[strum(serialize = "Allow")]
	Allow = 1,
}

impl WitnessAction {
	pub fn from_byte(byte: u8) -> Option<Self> {
		match byte {
			0 => Some(WitnessAction::Deny),
			1 => Some(WitnessAction::Allow),
			_ => None,
		}
	}
}
