// script_reader

use crate::core::{error::BuilderError, script::interop_service::InteropService};
use neo_codec::Decoder;
use neo_types::{
	op_code::{OpCode, OperandSize},
	Bytes,
};
use rustc_serialize::hex::ToHex;
use std::hash::Hash;
use tokio::io::AsyncReadExt;

pub struct ScriptReader;

impl ScriptReader {
	pub fn get_interop_service_code(_hash: String) -> Option<InteropService> {
		InteropService::from_hash(_hash)
	}
	pub fn convert_to_op_code_string(script: &Bytes) -> String {
		let mut reader = Decoder::new(script);
		let mut result = String::new();
		while reader.pointer().clone() < script.len() {
			if let Ok(op_code) = OpCode::try_from(reader.read_u8()) {
				result.push_str(&format!("{:?}", op_code).to_uppercase());
				if let Some(size) = op_code.operand_size() {
					if size.size().clone() > 0 {
						result.push_str(&format!(
							" {}",
							reader.read_bytes(size.size().clone() as usize).unwrap().to_hex()
						));
					} else if size.prefix_size().clone() > 0 {
						let prefix_size = Self::get_prefix_size(&mut reader, size).unwrap();
						result.push_str(&format!(
							" {} {}",
							prefix_size,
							reader.read_bytes(prefix_size).unwrap().to_hex()
						));
					}
				}
				result.push('\n');
			}
		}
		result
	}

	fn get_prefix_size(reader: &mut Decoder, size: OperandSize) -> Result<usize, BuilderError> {
		match size.prefix_size() {
			1 => Ok(reader.read_u8() as usize),
			2 => Ok(reader.read_i16() as usize),
			4 => Ok(reader.read_i32() as usize),
			_ => Err(BuilderError::UnsupportedOperation(
				"Only operand prefix sizes 1, 2, and 4 are supported".to_string(),
			)),
		}
	}
}
