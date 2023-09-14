
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractParameterType {
    Any = 0x00,
    Boolean = 0x10,
    Integer = 0x11,
    ByteArray = 0x12,
    String = 0x13,
    H160 = 0x14,
    H256 = 0x15,
    PublicKey = 0x16,
    Signature = 0x17,
    Array = 0x20,
    Map = 0x22,
    InteropInterface = 0x30,
    Void = 0xff,
}

impl ContractParameterType {

    pub fn as_str(&self) -> &str {
        match self {
            Self::Any => "Any",
            Self::Boolean => "Boolean",
            Self::Integer => "Integer",
            Self::ByteArray => "ByteArray",
            Self::String => "String",
            Self::H160 => "H160",
            Self::H256 => "H256",
            Self::PublicKey => "PublicKey",
            Self::Signature => "Signature",
            Self::Array => "Array",
            Self::Map => "Map",
            Self::InteropInterface => "InteropInterface",
            Self::Void => "Void",
        }
    }

    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::Any),
            0x10 => Some(Self::Boolean),
            0x11 => Some(Self::Integer),
            0x12 => Some(Self::ByteArray),
            0x13 => Some(Self::String),
            0x14 => Some(Self::H160),
            0x15 => Some(Self::H256),
            0x16 => Some(Self::PublicKey),
            0x17 => Some(Self::Signature),
            0x20 => Some(Self::Array),
            0x22 => Some(Self::Map),
            0x30 => Some(Self::InteropInterface),
            0xff => Some(Self::Void),
            _ => None,
        }
    }

}