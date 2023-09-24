use crate::{
	contract::contract_error::ContractError,
	crypto::hash::HashableForVec,
	protocol::core::stack_item::StackItem,
	serialization::binary_reader::BinaryReader,
	types::{contract_parameter::ContractParameter, Bytes},
	utils::*,
};
use p256::pkcs8::der::Encode;
use primitive_types::H160;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::hash::Hasher;
use tokio::io::AsyncReadExt;

const MAGIC: u32 = 0x3346454E;
const MAGIC_SIZE: usize = 4;
const COMPILER_SIZE: usize = 64;
const MAX_SOURCE_URL_SIZE: usize = 256;
const MAX_SCRIPT_LENGTH: usize = 512 * 1024;
const CHECKSUM_SIZE: usize = 4;
pub const HEADER_SIZE: usize = MAGIC_SIZE + COMPILER_SIZE;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NefFile {
	#[serde(skip_serializing_if = "Option::is_none")]
	compiler: Option<String>,
	source_url: String,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	#[serde(serialize_with = "serialize_vec_methodtoken")]
	#[serde(deserialize_with = "deserialize_vec_methodtoken")]
	method_tokens: Vec<MethodToken>,
	script: Bytes,
	checksum: Bytes,
}

impl Into<ContractParameter> for NefFile {
	fn into(self) -> ContractParameter {
		ContractParameter::string(serde_json::to_string(&self).unwrap())
	}
}

impl NefFile {
	fn get_checksum_as_integer(bytes: &Bytes) -> i32 {
		let mut bytes = bytes.clone();
		bytes.reverse();
		i32::from_be_bytes(bytes.try_into().unwrap())
	}

	fn compute_checksum(file: &NefFile) -> Bytes {
		Self::compute_checksum_from_bytes(serde_json::to_vec(file).unwrap())
	}

	fn compute_checksum_from_bytes(bytes: Bytes) -> Bytes {
		let mut file_bytes = bytes.clone();
		file_bytes.truncate(bytes.len() - CHECKSUM_SIZE);
		file_bytes.hash256()[..CHECKSUM_SIZE].try_into().unwrap()
	}

	fn read_from_file(file: &str) -> Result<Self, ContractError> {
		let file_bytes = std::fs::read(file).unwrap();
		if file_bytes.len() > 0x100000 {
			return Err(ContractError::InvalidArgError("NEF file is too large".to_string()))
		}

		let mut reader = BinaryReader::new(&file_bytes);
		let nef = reader.read_serializable().unwrap();
		Ok(nef)
	}

	fn read_from_stack_item(item: StackItem) -> Result<Self, ContractError> {
		if let StackItem::ByteString { value: bytes } = item {
			let mut reader = BinaryReader::new(&bytes.as_bytes());
			let nef = reader.read_serializable().unwrap();
			Ok(nef)
		} else {
			Err(ContractError::UnexpectedReturnType(
				item.to_json().unwrap() + StackItem::BYTE_STRING_VALUE,
			))
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodToken {
	#[serde(deserialize_with = "deserialize_address")]
	#[serde(serialize_with = "serialize_address")]
	hash: H160,
	method: String,
	params_count: u16,
	has_return_value: bool,
	call_flags: u8,
}

impl MethodToken {
	const PARAMS_COUNT_SIZE: usize = 2;
	const HAS_RETURN_VALUE_SIZE: usize = 1;
	const CALL_FLAGS_SIZE: usize = 1;
}
