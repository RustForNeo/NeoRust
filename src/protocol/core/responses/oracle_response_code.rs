use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Copy, Clone)]
pub enum OracleResponseCode {
    Success = 0x00,
    ProtocolNotSupported = 0x10,
    ConsensusUnreachable = 0x12,
    NotFound = 0x14,
    Timeout = 0x16,
    Forbidden = 0x18,
    ResponseTooLarge = 0x1A,
    InsufficientFunds = 0x1C,
    ContentTypeNotSupported = 0x1F,
    Error = 0xFF,
}

impl OracleResponseCode {

    pub fn as_str(&self) -> &str {
        match self {
            OracleResponseCode::Success => "Success",
            OracleResponseCode::ProtocolNotSupported => "ProtocolNotSupported",
            OracleResponseCode::ConsensusUnreachable => "ConsensusUnreachable",
            OracleResponseCode::NotFound => "NotFound",
            OracleResponseCode::Timeout => "Timeout",
            OracleResponseCode::Forbidden => "Forbidden",
            OracleResponseCode::ResponseTooLarge => "ResponseTooLarge",
            OracleResponseCode::InsufficientFunds => "InsufficientFunds",
            OracleResponseCode::ContentTypeNotSupported => "ContentTypeNotSupported",
            OracleResponseCode::Error => "Error",
        }
    }

}