use num_enum::FromPrimitive;
use strum_macros::{Display, EnumString};

#[derive(Display, EnumString, Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum Role {
	#[strum(serialize = "StateValidator")]
	StateValidator = 0x04,
	#[strum(serialize = "Oracle")]
	Oracle = 0x08,
	#[strum(serialize = "NeoFSAlphabetNode")]
	NeoFsAlphabetNode = 0x10,
}

impl Role {
	pub fn byte_repr(self) -> u8 {
		self as u8
	}
}
