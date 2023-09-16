// script_reader

use std::error::Error;
use crate::neo_error::NeoError;
use crate::script::interop_service::InteropService;
use crate::script::op_code::{OpCode, OperandSize};
use crate::serialization::binary_reader::BinaryReader;
use crate::types::Bytes;
use crate::utils::bytes::BytesExtern;

pub struct ScriptReader;

impl ScriptReader {

    pub fn get_interop_service_code(_hash: String) -> Option<InteropService> {
        InteropService::iter().find(|service| service.hash() == _hash)
    }

    pub fn convert_to_op_code_string(script: &Bytes) -> String {
        let mut reader = BinaryReader::new(script);
        let mut result = String::new();
        while reader.position() < script.len() {
            if let Some(op_code) = OpCode::from_u8(reader.read_u8()) {
                result.push_str(&format!("{}", op_code).to_uppercase());
                if let Some(size) = op_code.operand_size() {
                    if size.size() > 0 {
                        result.push_str(&format!(" {}", reader.read_bytes(size.size()).unwrap().to_hex()));
                    } else if size.prefix_size() > 0 {
                        let prefix_size = Self::get_prefix_size(&mut reader, size).unwrap();
                        result.push_str(&format!(" {} {}", prefix_size, reader.read_bytes(prefix_size).unwrap().to_hex()));
                    }
                }
                result.push('\n');
            }
        }
        result
    }

    fn get_prefix_size(reader: &mut BinaryReader, size: OperandSize) -> Result<usize, dyn Error> {
        match size.prefix_size() {
            1 => Ok(reader.read_u8() as usize),
            2 => Ok(reader.read_i16() as usize),
            4 => Ok(reader.read_i32() as usize),
            _ => Err(NeoError::UnsupportedOperation(
                "Only operand prefix sizes 1, 2, and 4 are supported".to_string(),
            )),
        }
    }


}