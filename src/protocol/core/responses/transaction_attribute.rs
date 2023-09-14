use std::hash::Hasher;
use serde::{Serialize, Deserialize};
use tokio::io::AsyncReadExt;
use crate::protocol::core::responses::oracle_response_code::OracleResponseCode;
use crate::serialization::binary_reader::BinaryReader;
use crate::serialization::binary_writer::BinaryWriter;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(tag = "type")]
pub enum TransactionAttribute {
    #[serde(rename = "HighPriority")]
    HighPriority,

    #[serde(rename = "OracleResponse")]
    OracleResponse {
        id: u32,
        code: OracleResponseCode,
        result: String,
    },
}

impl TransactionAttribute {

    pub fn serialize(&self, writer: &mut BinaryWriter) {
        match self {
            TransactionAttribute::HighPriority => {
                writer.write_u8(0x01);
            },
            TransactionAttribute::OracleResponse { id, code, result } => {
                writer.write_u8(0x11);
                writer.write_u32(*id);
                writer.write_u8(code.to_u8());
                writer.write_var_bytes(result.as_bytes());
            }
        }
    }

    pub fn deserialize(reader: &mut BinaryReader) -> Result<Self, &'static str> {
        match reader.read_u8()? {
            0x01 => Ok(Self::HighPriority),
            0x11 => {
                let id = reader.read_u32()?;
                let code = OracleResponseCode::from_u8(reader.read_u8()?)?;
                let result = reader.read_var_bytes()?.to_vec();
                Ok(Self::OracleResponse {
                    id,
                    code,
                    result: String::from_utf8(result).map_err(|_| "Invalid UTF8")?,
                })
            }
            _ => Err("Invalid attribute type"),
        }
    }

}