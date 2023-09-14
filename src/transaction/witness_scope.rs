#[derive(Debug, PartialEq, Eq)]
pub enum WitnessScope {
    None = 0x00,
    CalledByEntry = 0x01,
    CustomContracts = 0x10,
    CustomGroups = 0x20,
    WitnessRules = 0x40,
    Global = 0x80,
}

impl WitnessScope {

    pub fn to_string(&self) -> &str {
        match self {
            WitnessScope::None => "None",
            WitnessScope::CalledByEntry => "CalledByEntry",
            WitnessScope::CustomContracts => "CustomContracts",
            WitnessScope::CustomGroups => "CustomGroups",
            WitnessScope::WitnessRules => "WitnessRules",
            WitnessScope::Global => "Global",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "None" => Some(WitnessScope::None),
            "CalledByEntry" => Some(WitnessScope::CalledByEntry),
            "CustomContracts" => Some(WitnessScope::CustomContracts),
            "CustomGroups" => Some(WitnessScope::CustomGroups),
            "WitnessRules" => Some(WitnessScope::WitnessRules),
            "Global" => Some(WitnessScope::Global),
            _ => None,
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