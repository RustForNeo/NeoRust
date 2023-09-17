use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WitnessAction {
	Deny = 0,
	Allow = 1,
}

impl WitnessAction {
	pub fn to_string(&self) -> &str {
		match self {
			WitnessAction::Deny => "Deny",
			WitnessAction::Allow => "Allow",
		}
	}

	pub fn from_byte(byte: u8) -> Option<Self> {
		match byte {
			0 => Some(WitnessAction::Deny),
			1 => Some(WitnessAction::Allow),
			_ => None,
		}
	}
}
