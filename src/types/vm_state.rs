use num_enum::FromPrimitive;
use strum_macros::{Display, EnumString};

#[derive(Display, EnumString, Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum VMState {
	#[strum(serialize = "NONE")]
	None = 0,
	#[strum(serialize = "HALT")]
	Halt = 1,
	#[strum(serialize = "FAULT")]
	Fault = 2,
	#[strum(serialize = "BREAK")]
	Break = 4,
}
