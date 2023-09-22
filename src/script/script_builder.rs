use crate::{
	neo_error::NeoError,
	script::{interop_service::InteropService, op_code::OpCode},
	serialization::binary_writer::BinaryWriter,
	types::{
		call_flags::CallFlags,
		contract_parameter::{ContractParameter, ParameterValue},
		contract_parameter_type::ContractParameterType,
		Bytes, H160Externsion, PublicKey,
	},
};
use p256::{elliptic_curve::sec1::ToEncodedPoint, pkcs8::der::Encode};
use primitive_types::H160;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;

#[derive(Debug, PartialEq, Eq, Hash, CopyGetters, Setters)]
#[getset(get_copy, set)]
#[derive(educe::Educe)]
// note `new` below: generate `new()` that calls Default
#[educe(Default(new))]
pub struct ScriptBuilder {
	pub script: BinaryWriter,
}

impl ScriptBuilder {
	pub fn op_code(&mut self, op_codes: &[OpCode]) -> &mut Self {
		for opcode in op_codes {
			self.script.write_u8(opcode.opcode());
		}
		self
	}

	pub fn op_code_with_arg(&mut self, opcode: OpCode, argument: Bytes) -> &mut Self {
		self.script.write_u8(opcode.opcode());
		let _ = self.script.write_bytes(&argument);
		self
	}

	pub fn contract_call(
		&mut self,
		hash160: &H160,
		method: &str,
		params: &[ContractParameter],
		call_flags: CallFlags,
	) -> Result<&mut Self, NeoError> {
		if params.is_empty() {
			self.op_code(&[OpCode::NewArray]);
		} else {
			self.push_params(params);
		}

		Ok(self
			.push_integer(call_flags.bits())
			.unwrap()
			.push_data(method.as_bytes().to_vec())
			.unwrap()
			.push_data(hash160.to_vec())
			.unwrap()
			.sys_call(InteropService::SystemContractCall))
	}

	pub fn sys_call(&mut self, operation: InteropService) -> &mut Self {
		self.op_code(&[OpCode::Syscall])
			.push_data(operation.hash().into_bytes())
			.unwrap()
	}

	pub fn push_params(&mut self, params: &[ContractParameter]) -> &mut Self {
		for param in params {
			self.push_param(param).unwrap();
		}

		self.push_integer(params.len() as i64).unwrap().op_code(&[OpCode::Pack])
	}

	pub fn push_param(&mut self, param: &ContractParameter) -> Result<&mut Self, NeoError> {
		if param.get_type() == ContractParameterType::Any {
			self.op_code(&[OpCode::PushNull]);
		}
		match &param.value.unwrap() {
			ParameterValue::Boolean(b) => self.push_bool(*b),
			ParameterValue::Integer(i) => self.push_integer(i.clone()).unwrap(),
			ParameterValue::ByteArray(b)
			| ParameterValue::Signature(b)
			| ParameterValue::PublicKey(b) => self.push_data(b.as_bytes().to_vec()).unwrap(),
			ParameterValue::Hash160(h) => self.push_data(h.as_bytes().to_vec()).unwrap(),
			ParameterValue::Hash256(h) => self.push_data(h.as_bytes().to_vec()).unwrap(),
			ParameterValue::String(s) => self.push_data(s.as_bytes().to_vec()).unwrap(),
			ParameterValue::Array(arr) => self.push_array(arr).unwrap(),
			ParameterValue::Map(map) => {
				// Create an empty HashMap to hold your ContractParameter key-value pairs
				let mut map_value: HashMap<ContractParameter, ContractParameter> = HashMap::new();

				// Iterate over pairs of elements in the vector
				// (assuming the vector has an even number of elements)
				for i in (0..map.len()).step_by(2) {
					let key: ContractParameter =
						serde_json::from_str(map[i].as_str().unwrap()).unwrap();
					let value: ContractParameter =
						serde_json::from_str(map[i + 1].as_str().unwrap()).unwrap();

					// Insert the key-value pair into the HashMap
					map_value.insert(key, value);
				}

				self.push_map(&map_value).unwrap()
			},
			_ => return Err(NeoError::IllegalArgument("Unsupported parameter type".to_string())),
		};

		Ok(self)
	}

	// Additional push_* methods

	pub fn push_integer(&mut self, n: i64) -> Result<&mut Self, NeoError> {
		if n == -1 {
			self.op_code(&[OpCode::PushM1]);
		} else if 0 <= n && n <= 16 {
			self.op_code(&[OpCode::try_from(OpCode::Push0 as u8 + n as u8).unwrap()]);
		} else {
			let mut bytes = n.to_be_bytes();
			match self.len() {
				1 => self.op_code_with_arg(OpCode::PushInt8, bytes.to_vec().unwrap()),
				2 => self.op_code_with_arg(OpCode::PushInt16, bytes.to_vec().unwrap()),
				4 => self.op_code_with_arg(OpCode::PushInt32, bytes.to_vec().unwrap()),
				8 => self.op_code_with_arg(OpCode::PushInt64, bytes.to_vec().unwrap()),
				16 => self.op_code_with_arg(OpCode::PushInt128, bytes.to_vec().unwrap()),
				32 => self.op_code_with_arg(OpCode::PushInt256, bytes.to_vec().unwrap()),
				_ => return Err(NeoError::NumericOverflow),
			};
		}
		Ok(self)
	}

	fn pad_number(&self, n: i128, size: usize) -> Bytes {
		let mut bytes = n.to_vec().unwrap(); // .to_signed_bytes();
		if self.len() == size {
			return bytes
		}
		let pad_byte = if n.is_negative() { 0xff } else { 0 };
		if n.is_negative() {
			let mut padding =
				Bytes::from_iter(std::iter::repeat(pad_byte).take(size - &self.len()));
			padding.append(&mut bytes);
			padding
		} else {
			let mut result = bytes;
			result.resize(size, pad_byte);
			result
		}
	}

	// Push data handling

	pub fn push_data(&mut self, data: Bytes) -> Result<&mut Self, NeoError> {
		match data.len() {
			0..=75 => {
				self.op_code(&[OpCode::PushData1]);
				self.script.write_u8(data.len() as u8);
				let _ = self.script.write_bytes(&data);
			},
			76..=0xff => {
				self.op_code(&[OpCode::PushData2]);
				self.script.write_u16(data.len() as u16);
				let _ = self.script.write_bytes(&data);
			},
			0x100..=0xffff => {
				self.op_code(&[OpCode::PushData4]);
				self.script.write_u32(data.len() as u32);
				let _ = self.script.write_bytes(&data);
			},
			_ => return Err(NeoError::IllegalArgument("Data too long".to_string())),
		}
		Ok(self)
	}

	pub fn push_bool(&mut self, b: bool) -> &mut Self {
		if b {
			self.op_code(&[OpCode::PushTrue])
		} else {
			self.op_code(&[OpCode::PushFalse])
		};
		self
	}

	pub fn push_array(&mut self, arr: &[ContractParameter]) -> Result<&mut Self, NeoError> {
		if arr.is_empty() {
			self.op_code(&[OpCode::NewArray]);
		} else {
			let arrr = arr
				.iter()
				.map(|v| {
					let vv: ContractParameter = v.clone().into();
					vv
				})
				.collect();
			self.push_params(&arrr);
		};
		Ok(self)
	}

	pub fn push_map(
		&mut self,
		map: &HashMap<ContractParameter, ContractParameter>,
	) -> Result<&mut Self, NeoError> {
		for (k, v) in map {
			let kk: ContractParameter = k.clone().into();
			let vv: ContractParameter = v.clone().into();
			self.push_param(&vv).unwrap();
			self.push_param(&kk).unwrap();
		}

		Ok(self.push_integer(map.len() as i64).unwrap().op_code(&[OpCode::PackMap]))
	}

	// Additional helper methods

	pub fn pack(&mut self) -> &mut Self {
		self.op_code(&[OpCode::Pack])
	}

	pub fn to_bytes(&self) -> Bytes {
		self.script.script()
	}

	pub fn build_verification_script(pub_key: &PublicKey) -> Bytes {
		let mut sb = ScriptBuilder::new();
		sb.push_data(pub_key.to_encoded_point(false).as_bytes().to_vec())
			.unwrap()
			.sys_call(InteropService::SystemCryptoCheckSig);
		sb.to_bytes()
	}

	pub fn build_multisig_script(
		pubkeys: &mut [PublicKey],
		threshold: u8,
	) -> Result<Bytes, NeoError> {
		let mut sb = ScriptBuilder::new();
		sb.push_integer(threshold as i64).unwrap();
		pubkeys.sort_by(|a, b| a.to_encoded_point(true).cmp(&b.to_encoded_point(true)));
		for pk in pubkeys.iter() {
			sb.push_data(pk.to_encoded_point(true).as_bytes().to_vec()).unwrap();
		}
		sb.push_integer(pubkeys.len() as i64).unwrap();
		sb.sys_call(InteropService::SystemCryptoCheckMultisig);
		Ok(sb.to_bytes())
	}

	pub fn build_contract_script(
		sender: &H160,
		nef_checksum: u32,
		name: &str,
	) -> Result<Bytes, NeoError> {
		let mut sb = ScriptBuilder::new();
		sb.op_code(&[OpCode::Abort])
			.push_data(sender.to_vec())
			.unwrap()
			.push_integer(nef_checksum as i64)
			.unwrap()
			.push_data(name.as_bytes().to_vec())
			.unwrap();
		Ok(sb.to_bytes())
	}
	pub fn build_contract_call_and_unwrap_iterator(
		contract_hash: &H160,
		method: &str,
		params: &[ContractParameter],
		max_items: u32,
		call_flags: CallFlags,
	) -> Result<Bytes, NeoError> {
		let mut sb = Self::new();
		sb.push_integer(max_items as i64).unwrap();

		sb.contract_call(contract_hash, method, params, call_flags).unwrap();

		sb.op_code(&[OpCode::NewArray]);

		let cycle_start = sb.len();
		sb.op_code(&[OpCode::Over]);
		sb.sys_call(InteropService::SystemIteratorNext);

		let jmp_if_not = sb.len();
		sb.op_code_with_arg(OpCode::JmpIf, vec![0]);

		sb.op_code(&[OpCode::Dup, OpCode::Push2, OpCode::Pick])
			.sys_call(InteropService::SystemIteratorValue)
			.op_code(&[
				OpCode::Append,
				OpCode::Dup,
				OpCode::Size,
				OpCode::Push3,
				OpCode::Pick,
				OpCode::Ge,
			]);

		let jmp_if_max = sb.len();
		sb.op_code_with_arg(OpCode::JmpIf, vec![0]);

		let jmp_offset = sb.len();
		let jmp_bytes = (cycle_start - jmp_offset) as u8;
		sb.op_code_with_arg(OpCode::Jmp, vec![jmp_bytes]);

		let load_result = sb.len();
		sb.op_code(&[OpCode::Nip, OpCode::Nip]);

		let mut script = sb.to_bytes();
		let jmp_not_bytes = (load_result - jmp_if_not) as i8;
		script[jmp_if_not + 1] = jmp_not_bytes as u8;

		let jmp_max_bytes = (load_result - jmp_if_max) as i8;
		script[jmp_if_max + 1] = jmp_max_bytes as u8;

		Ok(script)
	}

	pub fn len(&self) -> usize {
		self.len()
	}
	// Other static helper methods
}
