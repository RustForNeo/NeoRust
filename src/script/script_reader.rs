// script_reader

use crate::script::{interop_service, op_code};
use crate::script::interop_service::InteropService;
use crate::serialization::binary_reader::BinaryReader;
use crate::types::Bytes;

pub struct ScriptReader;

impl ScriptReader {

    pub fn get_interop_service_code(hash: &str) -> Option<InteropService> {
        // match hash to service
    }

    pub fn convert_to_op_code_string(script: Bytes) -> String {
        let mut reader = BinaryReader::new(&script);
        let mut result = String::new();

        while !reader.is_eof() {
            let op = reader.read_u8().unwrap();
            let op = op_code::from(op);

            result.push_str(&op.to_string());

            if let Some(size) = op.operand_size() {
                // read and append operand
            }

            result.push('\n');
        }

        result
    }

}