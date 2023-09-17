#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Role {
	StateValidator = 0x04,
	Oracle = 0x08,
	NeoFsAlphabetNode = 0x10,
}

impl Role {
	pub fn to_string(&self) -> &str {
		match self {
			Role::StateValidator => "StateValidator",
			Role::Oracle => "Oracle",
			Role::NeoFsAlphabetNode => "NeoFSAlphabetNode",
		}
	}

	pub fn from_byte(byte: u8) -> Option<Self> {
		match byte {
			0x04 => Some(Role::StateValidator),
			0x08 => Some(Role::Oracle),
			0x10 => Some(Role::NeoFsAlphabetNode),
			_ => None,
		}
	}

	pub fn byte_repr(self) -> u8 {
		self as u8
	}
}
