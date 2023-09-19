use strum_macros::{Display, EnumString};

#[derive(Display, EnumString, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum WitnessScope {
	#[strum(serialize = "None")]
	None = 0x00,
	#[strum(serialize = "CalledByEntry")]
	CalledByEntry = 0x01,
	#[strum(serialize = "CustomContracts")]
	CustomContracts = 0x10,
	#[strum(serialize = "CustomGroups")]
	CustomGroups = 0x20,
	#[strum(serialize = "WitnessRules")]
	WitnessRules = 0x40,
	#[strum(serialize = "Global")]
	Global = 0x80,
}

impl WitnessScope {
	pub fn from_str(s: &str) -> Self {
		match s.parse::<WitnessScope>() {
			Ok(scope) => scope,
			Err(_) => panic!("Invalid witness scope: {}", s),
		}
	}

	pub fn byte_repr(&self) -> u8 {
		match self {
			WitnessScope::None => 0x00,
			WitnessScope::CalledByEntry => 0x01,
			WitnessScope::CustomContracts => 0x10,
			WitnessScope::CustomGroups => 0x20,
			WitnessScope::WitnessRules => 0x40,
			WitnessScope::Global => 0x80,
		}
	}

	pub fn from_byte(byte: u8) -> Option<Self> {
		match byte {
			0x00 => Some(WitnessScope::None),
			0x01 => Some(WitnessScope::CalledByEntry),
			0x10 => Some(WitnessScope::CustomContracts),
			0x20 => Some(WitnessScope::CustomGroups),
			0x40 => Some(WitnessScope::WitnessRules),
			0x80 => Some(WitnessScope::Global),
			_ => None,
		}
	}

	pub fn combine(scopes: &[Self]) -> u8 {
		let mut flags = 0;
		for scope in scopes {
			flags |= scope.byte_repr();
		}
		flags
	}

	// Split bit flags
	pub fn split(flags: u8) -> Vec<Self> {
		let mut scopes = Vec::new();

		if flags & Self::None.byte_repr() != 0 {
			scopes.push(Self::None);
		}
		if flags & Self::CalledByEntry.byte_repr() != 0 {
			scopes.push(Self::CalledByEntry);
		}
		if flags & Self::CustomContracts.byte_repr() != 0 {
			scopes.push(Self::CustomContracts);
		}
		if flags & Self::CustomGroups.byte_repr() != 0 {
			scopes.push(Self::CustomGroups);
		}
		if flags & Self::WitnessRules.byte_repr() != 0 {
			scopes.push(Self::WitnessRules);
		}
		if flags & Self::Global.byte_repr() != 0 {
			scopes.push(Self::Global);
		}

		scopes
	}
}
