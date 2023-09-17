#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VMState {
	None,
	Halt,
	Fault,
	Break,
}

impl VMState {
	pub fn as_str(&self) -> &str {
		match self {
			Self::None => "NONE",
			Self::Halt => "HALT",
			Self::Fault => "FAULT",
			Self::Break => "BREAK",
		}
	}

	pub fn from_str(s: &str) -> Option<Self> {
		match s {
			"NONE" => Some(Self::None),
			"HALT" => Some(Self::Halt),
			"FAULT" => Some(Self::Fault),
			"BREAK" => Some(Self::Break),
			_ => None,
		}
	}

	pub fn to_int(&self) -> u8 {
		match self {
			Self::None => 0,
			Self::Halt => 1,
			Self::Fault => 1 << 1,
			Self::Break => 1 << 2,
		}
	}

	pub fn from_int(i: u8) -> Option<Self> {
		match i {
			0 => Some(Self::None),
			1 => Some(Self::Halt),
			2 => Some(Self::Fault),
			4 => Some(Self::Break),
			_ => None,
		}
	}
}
