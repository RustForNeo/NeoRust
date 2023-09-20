use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(
	Display,
	EnumString,
	TryFromPrimitive,
	IntoPrimitive,
	Serialize,
	Deserialize,
	PartialEq,
	Eq,
	Copy,
	Clone,
)]
#[repr(u8)]
pub enum OracleResponseCode {
	#[strum(serialize = "Success")]
	Success = 0x00,
	#[strum(serialize = "ProtocolNotSupported")]
	ProtocolNotSupported = 0x10,
	#[strum(serialize = "ConsensusUnreachable")]
	ConsensusUnreachable = 0x12,
	#[strum(serialize = "NotFound")]
	NotFound = 0x14,
	#[strum(serialize = "Timeout")]
	Timeout = 0x16,
	#[strum(serialize = "Forbidden")]
	Forbidden = 0x18,
	#[strum(serialize = "ResponseTooLarge")]
	ResponseTooLarge = 0x1A,
	#[strum(serialize = "InsufficientFunds")]
	InsufficientFunds = 0x1C,
	#[strum(serialize = "ContentTypeNotSupported")]
	ContentTypeNotSupported = 0x1F,
	#[strum(serialize = "Error")]
	Error = 0xFF,
}
//
// impl From<u8> for OracleResponseCode {
// 	fn from(byte: u8) -> Self {
// 		match byte {
// 			0x00 => OracleResponseCode::Success,
// 			0x10 => OracleResponseCode::ProtocolNotSupported,
// 			0x12 => OracleResponseCode::ConsensusUnreachable,
// 			0x14 => OracleResponseCode::NotFound,
// 			0x16 => OracleResponseCode::Timeout,
// 			0x18 => OracleResponseCode::Forbidden,
// 			0x1A => OracleResponseCode::ResponseTooLarge,
// 			0x1C => OracleResponseCode::InsufficientFunds,
// 			0x1F => OracleResponseCode::ContentTypeNotSupported,
// 			0xFF => OracleResponseCode::Error,
// 			_ => panic!("Invalid OracleResponseCode byte"),
// 		}
// 	}
// }
