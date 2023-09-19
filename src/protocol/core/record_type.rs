use num_enum::FromPrimitive;
use strum_macros::{AsRefStr, Display, EnumCount, EnumIter, EnumString, IntoStaticStr};

// #[derive(
// Debug,
// PartialEq,
// strum::Display,
// strum::IntoStaticStr,
// strum::AsRefStr,
// strum::EnumString,
// strum::EnumCount,
// strum::EnumIter,
// )]

#[derive(
	EnumString,
	IntoStaticStr,
	AsRefStr,
	EnumString,
	EnumCount,
	EnumIter,
	Display,
	Copy,
	Clone,
	Debug,
	PartialEq,
	Eq,
	FromPrimitive,
)]
pub enum RecordType {
	#[strum(serialize = "A")]
	A = 1,
	#[strum(serialize = "CNAME")]
	CNAME = 5,
	#[strum(serialize = "TXT")]
	TXT = 16,
	#[strum(serialize = "AAAA")]
	AAAA = 28,
}

impl RecordType {
	pub fn byte_repr(self) -> u8 {
		self as u8
	}
}
