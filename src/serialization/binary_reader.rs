use crate::{neo_error::NeoError, transaction::signer::Signer};
use num_bigint::{BigInt, Sign};
use p256::{elliptic_curve::sec1::FromEncodedPoint, EncodedPoint, ProjectivePoint};
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct BinaryReader<'a> {
	data: &'a [u8],
	position: usize,
	marker: usize,
}

impl<'a> BinaryReader<'a> {
	pub fn new(data: &'a [u8]) -> Self {
		Self { data, position: 0, marker: 0 }
	}

	pub fn read_bool(&mut self) -> bool {
		let val = self.data[self.position] == 1;
		self.position += 1;
		val
	}

	pub fn read_u8(&mut self) -> u8 {
		let val = self.data[self.position];
		self.position += 1;
		val
	}

	pub fn read_u16(&mut self) -> u16 {
		let bytes = self.read_bytes(2).unwrap();
		u16::from_ne_bytes(bytes.try_into().unwrap())
	}

	pub fn read_i16(&mut self) -> i16 {
		let bytes = self.read_bytes(2).unwrap();
		i16::from_ne_bytes(bytes.try_into().unwrap())
	}

	pub fn read_u32(&mut self) -> u32 {
		let bytes = self.read_bytes(4).unwrap();
		u32::from_ne_bytes(bytes.try_into().unwrap())
	}

	pub fn read_i32(&mut self) -> i32 {
		let bytes = self.read_bytes(4).unwrap();
		i32::from_ne_bytes(bytes.try_into().unwrap())
	}

	pub fn read_u64(&mut self) -> u64 {
		let bytes = self.read_bytes(8).unwrap();
		u64::from_ne_bytes(bytes.try_into().unwrap())
	}

	pub fn read_i64(&mut self) -> i64 {
		let bytes = self.read_bytes(8).unwrap();
		i64::from_ne_bytes(bytes.try_into().unwrap())
	}

	pub fn read_u128(&mut self) -> u128 {
		let bytes = self.read_bytes(16).unwrap();
		u128::from_ne_bytes(bytes.try_into().unwrap())
	}

	pub fn read_bigint(&mut self) -> Result<BigInt, NeoError> {
		let byte = self.read_u8();

		let negative = byte & 0x80 != 0;
		let len = match byte {
			0..=0x4b => 1,
			0x4c => self.read_u8() as usize,
			0x4d => self.read_u16() as usize,
			0x4e => self.read_u32() as usize,
			_ => return Err(NeoError::InvalidFormat),
		};

		let mut bytes = self.read_bytes(len).unwrap();
		if negative {
			// Flip sign bit
			if let Some(byte) = bytes.get_mut(len - 1) {
				*byte ^= 0x80;
			} else {
				return Err(NeoError::InvalidFormat)
			}
			// bytes.get_mut()[len - 1] ^= 0x80;
		}
		//TODO:: need to check be or le and sign
		Ok(BigInt::from_bytes_be(Sign::Minus, bytes))
	}
	pub fn read_i128(&mut self) -> i128 {
		let bytes = self.read_bytes(16).unwrap();
		i128::from_ne_bytes(bytes.try_into().unwrap())
	}

	pub fn read_encoded_ec_point(&mut self) -> Result<&'a [u8], &'static str> {
		let byte = self.read_u8();
		match byte {
			0x02 | 0x03 => Ok(self.read_bytes(32).unwrap()),
			_ => Err("Invalid encoded EC point"),
		}
	}

	// Other primitive reader methods

	pub fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], NeoError> {
		let start = self.position;
		self.position += count;
		self.data
			.get(start..self.position)
			.ok_or_else(|| NeoError::IndexOutOfBounds("Out of bounds".to_string()))
	}

	pub fn read_var_bytes(&mut self) -> Result<&'a [u8], NeoError> {
		let len = self.read_var_int().unwrap() as usize;
		self.read_bytes(len)
	}

	pub fn read_var_int(&mut self) -> Result<i64, NeoError> {
		let first = self.read_u8();
		match first {
			0xfd => Ok(self.read_u16() as i64),
			0xfe => Ok(self.read_u32() as i64),
			0xff => Ok(self.read_u64() as i64),
			_ => Ok(first as i64),
		}
	}

	pub fn read_string(&mut self) -> Result<String, NeoError> {
		let bytes = self.read_var_bytes().unwrap();

		let string = match String::from_utf8(bytes.to_vec()) {
			Ok(s) => s,
			Err(e) => {
				// Handle invalid UTF-8
				return Err(NeoError::InvalidEncoding(e.to_string()))
			},
		};

		// Trim null bytes from end
		let string = string.trim_end_matches(char::from(0));

		Ok(string.to_string())
	}

	pub fn read_push_bytes(&mut self) -> Result<&'a [u8], NeoError> {
		let opcode = self.read_u8();
		let len = match opcode {
			0x01..=0x4B => opcode as usize,
			0x4C => self.read_u8() as usize,
			0x4D => self.read_u16() as usize,
			0x4E => self.read_u32() as usize,
			_ => return Err(NeoError::InvalidOpCode),
		};

		self.read_bytes(len)
	}

	pub fn read_push_int(&mut self) -> Result<i64, NeoError> {
		let opcode = self.read_u8();
		match opcode {
			0x00..=0x16 => Ok(opcode as i64 - 1),
			0x01..=0x04 => {
				let n = match opcode {
					0x51 => 1,
					0x52 => 2,
					0x53 => 4,
					0x54 => 8,
					_ => {},
				};
				let bytes = self.read_bytes(n).unwrap();
				Ok(i64::from_be_bytes(bytes.try_into().unwrap()))
			},
			_ => Err(NeoError::InvalidOpCode),
		}
	}

	pub fn read_push_string(&mut self) -> Result<String, NeoError> {
		let bytes = self.read_push_bytes().unwrap();
		String::from_utf8(Vec::from(bytes))
			.map_err(|_| NeoError::InvalidEncoding("Invalid UTF-8".to_string()))
	}

	// Serialization helper methods

	pub fn read_serializable<T: Deserialize<'a>>(&mut self) -> Result<T, NeoError> {
		T::deserialize(self)
	}

	pub fn read_serializable_list<T: Deserialize<'a>>(&mut self) -> Result<Vec<Signer>, NeoError> {
		let len = self.read_var_int().unwrap();
		let mut list = Vec::with_capacity(len as usize);
		for _ in 0..len {
			list.push(self.read_serializable().unwrap());
		}
		Ok(list)
	}

	// Other methods like `mark`, `reset`, etc.

	pub fn mark(&mut self) {
		self.marker = self.position;
	}

	pub fn reset(&mut self) {
		self.position = self.marker;
	}

	pub fn read_ec_point(&mut self) -> Result<ProjectivePoint, &'static str> {
		let tag = self.read_byte();
		let bytes = match tag {
			0x00 => return Ok(ProjectivePoint::IDENTITY),
			0x02 | 0x03 => self.read_bytes(32),
			0x04 => self.read_bytes(64),
			_ => return Err("Invalid EC point tag"),
		};

		let point = EncodedPoint::from_bytes(bytes);
		match ProjectivePoint::from_encoded_point(&point) {
			Some(point) => Ok(point),
			None => Err("Invalid EC point"),
		}
	}
}
