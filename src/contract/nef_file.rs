use crate::{
	contract::contract_error::ContractError,
	crypto::hash::HashableForVec,
	protocol::core::stack_item::StackItem,
	serialization::{binary_reader::BinaryReader, binary_writer::BinaryWriter},
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
const HEADER_SIZE: usize = MAGIC_SIZE + COMPILER_SIZE;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
		Self::compute_checksum_from_bytes(file.to_vec().unwrap())
	}

	fn compute_checksum_from_bytes(bytes: Bytes) -> Bytes {
		let mut file_bytes = bytes;
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
		if let StackItem::ByteString(bytes) = item {
			let mut reader = BinaryReader::new(&bytes);
			let nef = reader.read_serializable().unwrap();
			Ok(nef)
		} else {
			Err(ContractError::UnexpectedReturnType(
				item.json_value() + StackItem::BYTE_STRING_VALUE.into_string(),
			))
		}
	}

	fn serialize(&self, writer: &mut BinaryWriter) {
		writer.write_u32(MAGIC);
		writer
			.write_fixed_string(&self.compiler, COMPILER_SIZE)
			.expect("Invalid compiler size");
		writer.write_var_string(&self.source_url);
		writer.write_u8(0);
		writer.write_serializable_var(&self.method_tokens);
		writer.write_u16(0);
		writer.write_var_bytes(&self.script);
		writer.write(&self.checksum);
	}

	fn deserialize(reader: &mut BinaryReader) -> Result<Self, ContractError> {
		let magic = reader.read_u32();
		if magic != MAGIC {
			return Err(ContractError::InvalidArgError("Invalid magic number".to_string()))
		}

		let compiler_bytes = reader.read_bytes(COMPILER_SIZE).unwrap();
		let compiler = String::from_utf8(compiler_bytes.trim_right(0).to_vec()).ok();

		let source_url = reader.read_var_string().unwrap();
		if source_url.len() > MAX_SOURCE_URL_SIZE {
			return Err(ContractError::InvalidArgError(format!(
				"Source URL too long. Max {} bytes",
				MAX_SOURCE_URL_SIZE
			)))
		}

		if reader.read_u8().unwrap() != 0 {
			return Err(ContractError::InvalidArgError("Expected reserved byte to be 0".to_string()))
		}

		let method_tokens = reader.read_serializable_list().unwrap();

		if reader.read_u16().unwrap() != 0 {
			return Err(ContractError::InvalidArgError(
				"Expected reserved bytes to be 0".to_string(),
			))
		}

		let script = reader.read_var_bytes().unwrap();
		if script.is_empty() {
			return Err(ContractError::InvalidArgError("Script cannot be empty".to_string()))
		}

		let mut nef = NefFile {
			compiler,
			source_url,
			method_tokens,
			script: script.to_vec(),
			checksum: Bytes::new(),
		};

		nef.checksum = Self::compute_checksum(&nef);

		let checksum = reader.read_bytes(CHECKSUM_SIZE).unwrap();
		if nef.checksum != checksum {
			return Err(ContractError::InvalidArgError("Invalid checksum".to_string()))
		}
		Ok(nef)
	}
}

#[derive(Debug, Serialize, Deserialize)]
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

	fn serialize(&self, writer: &mut BinaryWriter) {
		writer.write_serializable(&self.hash);
		writer.write_var_string(&self.method);
		writer.write_u16(self.params_count);
		writer.write_bool(self.has_return_value);
		writer.write_u8(self.call_flags);
	}

	fn deserialize(reader: &mut BinaryReader) -> Result<Self, ContractError> {
		let hash = reader.read_serializable().unwrap();
		let method = reader.read_var_string().unwrap();
		let params_count = reader.read_u16().unwrap();
		let has_return_value = reader.read_bool().unwrap();
		let call_flags = reader.read_u8().unwrap();

		Ok(MethodToken { hash, method, params_count, has_return_value, call_flags })
	}
}
