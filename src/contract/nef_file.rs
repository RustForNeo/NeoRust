use crate::{
	contract::contract_error::ContractError,
	protocol::core::stack_item::StackItem,
	serialization::{binary_reader::BinaryReader, binary_writer::BinaryWriter},
	types::Bytes,
};
use p256::pkcs8::der::Encode;
use primitive_types::H160;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::hash::Hasher;
use tokio::io::AsyncReadExt;
use crate::crypto::hash::HashableForVec;

const MAGIC: u32 = 0x3346454E;
const MAGIC_SIZE: usize = 4;
const COMPILER_SIZE: usize = 64;
const MAX_SOURCE_URL_SIZE: usize = 256;
const MAX_SCRIPT_LENGTH: usize = 512 * 1024;
const CHECKSUM_SIZE: usize = 4;
const HEADER_SIZE: usize = MAGIC_SIZE + COMPILER_SIZE;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NefFile {
	compiler: Option<String>,
	source_url: String,
	method_tokens: Vec<MethodToken>,
	script: Bytes,
	checksum: Bytes,
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
		let file_bytes = std::fs::read(file)?;
		if file_bytes.len() > 0x100000 {
			return Err(ContractError::InvalidArgError("NEF file is too large".to_string()));
		}

		let mut reader = BinaryReader::new(&file_bytes);
		let nef = reader.read_serializable()?;
		Ok(nef)
	}

	fn read_from_stack_item(item: StackItem) -> Result<Self, ContractError> {
		if let StackItem::ByteString(bytes) = item {
			let mut reader = BinaryReader::new(&bytes);
			let nef = reader.read_serializable()?;
			Ok(nef)
		} else {
			Err(ContractError::UnexpectedReturnType(
				item.json_value(),
				Some(vec![StackItem::BYTE_STRING_VALUE.into_string()]),
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
			return Err(ContractError::InvalidArgError("Invalid magic number".to_string()));
		}

		let compiler_bytes = reader.read_bytes(COMPILER_SIZE)?;
		let compiler = String::from_utf8(compiler_bytes.trim_right(0).to_vec()).ok();

		let source_url = reader.read_var_string()?;
		if source_url.len() > MAX_SOURCE_URL_SIZE {
			return Err(ContractError::InvalidArgError(format!(
				"Source URL too long. Max {} bytes",
				MAX_SOURCE_URL_SIZE
			)));
		}

		if reader.read_u8()? != 0 {
			return Err(ContractError::InvalidArgError(
				"Expected reserved byte to be 0".to_string(),
			));
		}

		let method_tokens = reader.read_serializable_list()?;

		if reader.read_u16()? != 0 {
			return Err(ContractError::InvalidArgError(
				"Expected reserved bytes to be 0".to_string(),
			));
		}

		let script = reader.read_var_bytes()?;
		if script.is_empty() {
			return Err(ContractError::InvalidArgError("Script cannot be empty".to_string()));
		}

		let mut nef = NefFile {
			compiler,
			source_url,
			method_tokens,
			script: script.to_vec(),
			checksum: Bytes::new(),
		};

		nef.checksum = Self::compute_checksum(&nef);

		let checksum = reader.read_bytes(CHECKSUM_SIZE)?;
		if nef.checksum != checksum {
			return Err(ContractError::InvalidArgError("Invalid checksum".to_string()));
		}
		Ok(nef)
	}
}

impl Serialize for NefFile {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		self.compiler.and_then(|v| serializer.serialize_str(v.as_str()).unwrap()); //is(|c| serializer.serialize_str(c.as_str()));
		serializer.serialize_str(&self.source_url)?;
		self.method_tokens.serialize(serializer)?;
		serializer.serialize_bytes(&self.script)?;
		serializer.serialize_bytes(&self.checksum)
	}
}

impl<'de> Deserialize<'de> for NefFile {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		#[serde(field_identifier, rename_all = "lowercase")]
		enum Field {
			Compiler,
			SourceUrl,
			MethodTokens,
			Script,
			Checksum,
		}

		struct NefFileVisitor;

		impl<'de> de::Visitor<'de> for NefFileVisitor {
			type Value = NefFile;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("struct NefFile")
			}

			fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
			where
				V: de::MapAccess<'de>,
			{
				let mut compiler: Option<String> = None;
				let mut source_url = "".to_string();
				let mut method_tokens = Vec::new();
				let mut script = Vec::new();
				let mut checksum = Vec::new();

				while let Some(key) = map.next_key()? {
					match key {
						Field::Compiler => {
							if compiler.is_some() {
								return Err(de::Error::duplicate_field("compiler"));
							}
							compiler = Some(map.next_value()?);
						},
						Field::SourceUrl => {
							if !source_url.is_empty() {
								return Err(de::Error::duplicate_field("source_url"));
							}
							source_url = map.next_value()?;
						},
						Field::MethodTokens => {
							if !method_tokens.is_empty() {
								return Err(de::Error::duplicate_field("method_tokens"));
							}
							method_tokens = map.next_value()?;
						},
						Field::Script => {
							if !script.is_empty() {
								return Err(de::Error::duplicate_field("script"));
							}
							script = map.next_value()?;
						},
						Field::Checksum => {
							if !checksum.is_empty() {
								return Err(de::Error::duplicate_field("checksum"));
							}
							checksum = map.next_value()?;
						},
					}
				}

				Ok(NefFile { compiler, source_url, method_tokens, script, checksum })
			}
		}

		deserializer.deserialize_map(NefFileVisitor)
	}
}

#[derive(Debug)]
pub struct MethodToken {
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
		let hash = reader.read_serializable()?;
		let method = reader.read_var_string()?;
		let params_count = reader.read_u16()?;
		let has_return_value = reader.read_bool()?;
		let call_flags = reader.read_u8()?;

		Ok(MethodToken { hash, method, params_count, has_return_value, call_flags })
	}
}

impl Serialize for MethodToken {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.method)?;
		serializer.serialize_u16(self.params_count)?;
		serializer.serialize_bool(self.has_return_value)?;
		serializer.serialize_u8(self.call_flags)
	}
}

impl<'de> Deserialize<'de> for MethodToken {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct MethodTokenVisitor;

		impl<'de> de::Visitor<'de> for MethodTokenVisitor {
			type Value = MethodToken;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("struct MethodToken")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				Ok(MethodToken {
					hash: Default::default(),
					method: v.to_string(),
					// Add default values
					params_count: 0,
					has_return_value: false,
					call_flags: 0,
				})
			}
		}

		deserializer.deserialize_str(MethodTokenVisitor)
	}
}
