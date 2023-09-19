use crate::{
	crypto::key_pair::KeyPair,
	neo_error::NeoError,
	script::{interop_service::InteropService, op_code::OpCode},
	serialization::binary_writer::BinaryWriter,
	types::{
		call_flags::CallFlags,
		contract_parameter::{ContractParameter, ParameterValue},
		Bytes,
	},
};
use futures::AsyncWriteExt;
use p256::{elliptic_curve::sec1::ToEncodedPoint, pkcs8::der::Encode, PublicKey};
use primitive_types::H160;
use std::{collections::HashMap, error::Error};

#[derive(Debug, PartialEq, Eq, Hash, CopyGetters, Setters, Default)]
#[getset(get_copy, set)]
#[derive(educe::Educe)]
// note `new` below: generate `new()` that calls Default
#[educe(Default(new))]
pub struct ScriptBuilder {
	writer: BinaryWriter,
}

impl ScriptBuilder {
	// pub fn new() -> Self {
	// 	Self { writer: BinaryWriter::new() }
	// }

	pub fn op_code(&mut self, op_codes: &[OpCode]) -> &mut Self {
		for opcode in op_codes {
			self.writer.write_u8(opcode.into());
		}
		self
	}

	pub fn op_code_with_arg(&mut self, opcode: OpCode, argument: Bytes) -> &mut Self {
		self.writer.write_u8(opcode.into());
		let _ = self.writer.write(&argument);
		self
	}

	pub fn contract_call(
		&mut self,
		hash160: &H160,
		method: &str,
		params: &[Option<ContractParameter>],
		call_flags: CallFlags,
	) -> Result<&mut Self, dyn Error> {
		if params.is_empty() {
			self.op_code(&[OpCode::NewArray]);
		} else {
			self.push_params(params)?;
		}

		self.push_integer(call_flags.bits())?
			.push_data(method.as_bytes().to_vec())
			.push_data(hash160.to_bytes())
			.sys_call(InteropService::SystemContractCall)
	}

	pub fn sys_call(&mut self, operation: InteropService) -> &mut Self {
		self.op_code(&[OpCode::Syscall]).push_data(operation.to_hash().as_bytes())?
	}

	pub fn push_params(&mut self, params: &[Option<ContractParameter>]) -> &mut Self {
		for param in params {
			self.push_param(param)?;
		}

		self.push_integer(params.len() as i64)?.op_code(&[OpCode::Pack])?
	}

	pub fn push_param(
		&mut self,
		param: &Option<ContractParameter>,
	) -> Result<&mut Self, dyn Error> {
		match param {
			None => self.op_code(&[OpCode::PushNull]),
			Some(param) => {
				match &param.value {
					ParameterValue::Boolean(b) => self.push_bool(*b),
					ParameterValue::Integer(i) => self.push_integer(i.clone())?,
					ParameterValue::ByteArray(b)
					| ParameterValue::Signature(b)
					| ParameterValue::PublicKey(b) => self.push_data(b.as_bytes().to_vec()),
					ParameterValue::Hash160(h) => self.push_data(h.as_bytes().to_vec()),
					ParameterValue::Hash256(h) => self.push_data(h.as_bytes().to_vec()),
					ParameterValue::String(s) => self.push_data(s.as_bytes().to_vec()),
					ParameterValue::Array(arr) => self.push_array(arr)?,
					ParameterValue::Map(map) => {
						// Create an empty HashMap to hold your ContractParameter key-value pairs
						let mut map: HashMap<ContractParameter, ContractParameter> = HashMap::new();

						// Iterate over pairs of elements in the vector
						// (assuming the vector has an even number of elements)
						for i in (0..map.len()).step_by(2) {
							let key = ContractParameter::from_json_value(map[i].clone());
							let value = ContractParameter::from_json_value(map[i + 1].clone());

							// Insert the key-value pair into the HashMap
							map.insert(key, value);
						}

						self.push_map(&map)?
					},
					_ =>
						return Err(Error::IllegalArgument("Unsupported parameter type".to_string())),
				}
			},
		}
		Ok(self)
	}

	// Additional push_* methods

	pub fn push_integer(&mut self, n: i64) -> Result<&mut Self, dyn Error> {
		if n == -1 {
			self.op_code(&[OpCode::PushM1]);
		} else if 0 <= n && n <= 16 {
			self.op_code(&[OpCode::from_u8(OpCode::Push0.into() + n as u8).unwrap()]);
		} else {
			let mut bytes = n.to_be_bytes();
			match self.writer.size() {
				1 => self.op_code_with_arg(OpCode::PushInt8, bytes.to_vec().unwrap()),
				2 => self.op_code_with_arg(OpCode::PushInt16, bytes.to_vec().unwrap()),
				4 => self.op_code_with_arg(OpCode::PushInt32, bytes.to_vec().unwrap()),
				8 => self.op_code_with_arg(OpCode::PushInt64, bytes.to_vec().unwrap()),
				16 => self.op_code_with_arg(OpCode::PushInt128, bytes.to_vec().unwrap()),
				32 => self.op_code_with_arg(OpCode::PushInt256, bytes.to_vec().unwrap()),
				_ => return Err(Error::NumericOverflow),
			};
		}
		Ok(self)
	}

	fn pad_number(&self, n: i128, size: usize) -> Bytes {
		let mut bytes = n.to_signed_bytes();
		if self.writer.size() == size {
			return bytes
		}
		let pad_byte = if n.is_negative() { 0xff } else { 0 };
		if n.is_negative() {
			let mut padding =
				Bytes::from_iter(std::iter::repeat(pad_byte).take(size - &self.writer.size()));
			padding.append(&mut bytes);
			padding
		} else {
			let mut result = bytes;
			result.resize(size, pad_byte);
			result
		}
	}

	// Push data handling

	pub fn push_data(&mut self, data: Bytes) -> Result<&mut Self, dyn Error> {
		match data.len() {
			0...75 => {
				self.op_code(&[OpCode::PushData1]);
				self.writer.write_u8(data.len() as u8);
				let _ = self.writer.write(&data);
			},
			76...0xff => {
				self.op_code(&[OpCode::PushData2]);
				self.writer.write_u16(data.len() as u16);
				let _ = self.writer.write(&data);
			},
			0x100...0xffff => {
				self.op_code(&[OpCode::PushData4]);
				self.writer.write_u32(data.len() as u32);
				let _ = self.writer.write(&data);
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
		}
		self
	}

	pub fn push_array(&mut self, arr: &[ContractParameter]) -> Result<&mut Self, dyn Error> {
		if arr.is_empty() {
			self.op_code(&[OpCode::NewArray])
		} else {
			let arrr = arr
				.iter()
				.map(|v| {
					let vv: ContractParameter = v.clone().into();
					vv
				})
				.collect();
			self.push_params(&Some(arrr))?;
		}
		Ok(self)
	}

	pub fn push_map(
		&mut self,
		map: &HashMap<ContractParameter, ContractParameter>,
	) -> Result<&mut Self, dyn Error> {
		for (k, v) in map {
			let kk: ContractParameter = k.clone().into();
			let vv: ContractParameter = v.clone().into();
			self.push_param(&Some(vv))?;
			self.push_param(&Some(kk))?;
		}

		Ok(self.push_integer(map.len() as i64)?.op_code(&[OpCode::PackMap]))
	}

	// Additional helper methods

	pub fn pack(&mut self) -> &mut Self {
		self.op_code(&[OpCode::Pack])
	}

	pub fn to_bytes(&self) -> Bytes {
		self.writer.into_bytes()
	}

	pub fn build_verification_script(pub_key: &PublicKey) -> Bytes {
		let mut sb = ScriptBuilder::new();
		sb.push_data(pub_key.to_encoded_point(false).as_bytes().to_vec())?
			.sys_call(InteropService::SystemCryptoCheckSig);
		sb.to_bytes()
	}

	pub fn build_multisig_script(pubkeys: &[KeyPair], threshold: u8) -> Result<Bytes, NeoError> {
		let mut sb = ScriptBuilder::new();
		sb.push_int(threshold as i64)?;
		for pk in pubkeys
			.iter()
			.sorted_by(|a, b| a.to_encoded_point(true).cmp(&b.to_encoded_point(true)))
		{
			sb.push_data(pk.to_encoded_point(true))?;
		}
		sb.push_int(pubkeys.len() as i64)?;
		sb.sys_call(InteropService::SystemCryptoCheckMultisig);
		Ok(sb.to_bytes())
	}

	pub fn build_contract_script(
		sender: &H160,
		nef_checksum: u32,
		name: &str,
	) -> Result<Bytes, NeoError> {
		let mut sb = ScriptBuilder::new();
		sb.op_code(&[OpCode::Abort])?
			.push_data(sender.to_array()?.as_slice())?
			.push_int(nef_checksum as i64)?
			.push_data(name.as_bytes())?;
		Ok(sb.to_bytes())
	}
	pub fn build_contract_call_and_unwrap_iterator(
		contract_hash: &H160,
		method: &str,
		params: &[Option<ContractParameter>],
		max_items: u32,
		call_flags: CallFlags,
	) -> Result<Bytes, NeoError> {
		let mut sb = Self::new();
		sb.push_int(max_items as i64)?;

		sb.contract_call(contract_hash, method, params, call_flags)?;

		sb.op_code(&[OpCode::NewArray])?;

		let cycle_start = sb.writer.size();
		sb.op_code(&[OpCode::Over])?;
		sb.sys_call(InteropService::SystemIteratorNext)?;

		let jmp_if_not = sb.writer.size();
		sb.op_code_arg(OpCode::JmpIf, &[0])?;

		sb.op_code(&[OpCode::Dup, OpCode::Push2, OpCode::Pick])
			.sys_call(InteropService::SystemIteratorValue)
			.op_code(&[
				OpCode::Append,
				OpCode::Dup,
				OpCode::Size,
				OpCode::Push3,
				OpCode::Pick,
				OpCode::Ge,
			])?;

		let jmp_if_max = sb.writer.size();
		sb.op_code_arg(OpCode::JmpIf, &[0])?;

		let jmp_offset = sb.writer.size();
		let jmp_bytes = (cycle_start - jmp_offset) as i8;
		sb.op_code_arg(OpCode::Jmp, &[jmp_bytes])?;

		let load_result = sb.writer.size();
		sb.op_code(&[OpCode::Nip, OpCode::Nip])?;

		let mut script = sb.to_bytes();
		let jmp_not_bytes = (load_result - jmp_if_not) as i8;
		script[jmp_if_not + 1] = jmp_not_bytes as u8;

		let jmp_max_bytes = (load_result - jmp_if_max) as i8;
		script[jmp_if_max + 1] = jmp_max_bytes as u8;

		Ok(script)
	}

	// Other static helper methods
}
