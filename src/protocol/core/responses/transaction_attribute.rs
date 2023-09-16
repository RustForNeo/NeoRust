use serde::{Serialize, Deserialize, Deserializer, Serializer};
use serde::__private::de::Content::ByteBuf;
use crate::protocol::core::responses::oracle_response_code::OracleResponseCode;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(tag = "type")]
pub enum TransactionAttribute {
    #[serde(rename = "HighPriority")]
    HighPriority,

    #[serde(rename = "OracleResponse")]
    OracleResponse(
        u32,
        OracleResponseCode,
        String,
    ),
}

impl TransactionAttribute {
    pub const MAX_RESULT_SIZE: usize = 0xffff;

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        match self {
            TransactionAttribute::HighPriority => {
                bytes.push(0x01);
            }
            TransactionAttribute::OracleResponse(id, response_code, result ) => {
                bytes.push(0x11);
                bytes.extend(&id.to_be_bytes());
                bytes.push(response_code.to_byte());
                bytes.extend(result.as_bytes());
            }
        }

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        match bytes[0] {
            0x01 => Ok(TransactionAttribute::HighPriority),
            0x11 => {
                if bytes.len() < 9 {
                    return Err("Not enough bytes for OracleResponse");
                }
                let id = u64::from_be_bytes([0; 8 - bytes[1..9].len()].concat(bytes[1..9].try_into().unwrap()));
                let response_code = OracleResponseCode::from(bytes[9]);
                let result = String::from_utf8(bytes[10..].to_vec()).map_err(|_| "Invalid UTF-8")?;

                Ok(TransactionAttribute::OracleResponse(
                    id as u32,
                    response_code,
                    result,
                ))
            }
            _ => Err("Invalid attribute type byte"),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Serialize for TransactionAttribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let bytes = self.to_bytes();
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> Deserialize<'de> for TransactionAttribute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let bytes = ByteBuf::deserialize(deserializer)?;
        TransactionAttribute::from_bytes(&bytes)
            .map_err(serde::de::Error::custom)
    }
}