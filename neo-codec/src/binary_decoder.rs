/// This module provides a binary decoder that can read various types of data from a byte slice.
///
/// # Examples
///
/// ```
/// use neo_codec::binary_decoder::Decoder;
///
/// let data = [0x01, 0x02, 0x03, 0x04];
/// let mut decoder = Decoder::new(&data);
///
/// assert_eq!(decoder.read_bool(), true);
/// assert_eq!(decoder.read_u8(), 2);
/// assert_eq!(decoder.read_u16(), 0x0403);
/// assert_eq!(decoder.read_i16(), 0x0403);
/// assert_eq!(decoder.read_u32(), 0x04030201);
/// assert_eq!(decoder.read_i32(), 0x04030201);
/// assert_eq!(decoder.read_u64(), 0x0807060504030201);
/// assert_eq!(decoder.read_i64(), 0x0807060504030201);
/// assert_eq!(decoder.read_u128(), 0x100f0e0d0c0b0a090807060504030201);
/// assert_eq!(decoder.read_i128(), 0x100f0e0d0c0b0a090807060504030201);
/// ```
use crate::CodecError;
use getset::{Getters, Setters};
use num_bigint::{BigInt, Sign};
use serde::Deserialize;
use serde_derive::Serialize;

/// A binary decoder that can read various types of data from a byte slice.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Getters, Setters)]
pub struct Decoder<'a> {
	data: &'a [u8],
	#[getset(get = "pub")]
	pointer: usize,
	marker: usize,
}

impl<'a> Iterator for Decoder<'a> {
	type Item = u8;

	/// Returns the next byte in the byte slice, or None if the end of the slice has been reached.
	fn next(&mut self) -> Option<Self::Item> {
		if self.pointer < self.data.len() {
			let val = self.data[self.pointer];
			self.pointer += 1;
			Some(val)
		} else {
			None
		}
	}
}

impl<'a> Decoder<'a> {
	/// Creates a new binary decoder that reads from the given byte slice.
	pub fn new(data: &'a [u8]) -> Self {
		Self { data, pointer: 0, marker: 0 }
	}

	/// Reads a boolean value from the byte slice.
	pub fn read_bool(&mut self) -> bool {
		let val = self.data[self.pointer] == 1;
		self.pointer += 1;
		val
	}

	/// Reads an unsigned 8-bit integer from the byte slice.
	pub fn read_u8(&mut self) -> u8 {
		let val = self.data[self.pointer];
		self.pointer += 1;
		val
	}

	/// Reads an unsigned 16-bit integer from the byte slice.
	pub fn read_u16(&mut self) -> u16 {
		let bytes = self.read_bytes(2).unwrap();
		u16::from_ne_bytes(bytes.try_into().unwrap())
	}

	/// Reads a signed 16-bit integer from the byte slice.
	pub fn read_i16(&mut self) -> i16 {
		let bytes = self.read_bytes(2).unwrap();
		i16::from_ne_bytes(bytes.try_into().unwrap())
	}

	/// Reads an unsigned 32-bit integer from the byte slice.
	pub fn read_u32(&mut self) -> u32 {
		let bytes = self.read_bytes(4).unwrap();
		u32::from_ne_bytes(bytes.try_into().unwrap())
	}

	/// Reads a signed 32-bit integer from the byte slice.
	pub fn read_i32(&mut self) -> i32 {
		let bytes = self.read_bytes(4).unwrap();
		i32::from_ne_bytes(bytes.try_into().unwrap())
	}

	/// Reads an unsigned 64-bit integer from the byte slice.
	pub fn read_u64(&mut self) -> u64 {
		let bytes = self.read_bytes(8).unwrap();
		u64::from_ne_bytes(bytes.try_into().unwrap())
	}

	/// Reads a signed 64-bit integer from the byte slice.
	pub fn read_i64(&mut self) -> i64 {
		let bytes = self.read_bytes(8).unwrap();
		i64::from_ne_bytes(bytes.try_into().unwrap())
	}

	/// Reads an unsigned 128-bit integer from the byte slice.
	pub fn read_u128(&mut self) -> u128 {
		let bytes = self.read_bytes(16).unwrap();
		u128::from_ne_bytes(bytes.try_into().unwrap())
	}

	/// Reads a signed big integer from the byte slice.
	pub fn read_bigint(&mut self) -> Result<BigInt, CodecError> {
		let byte = self.read_u8();

		let negative = byte & 0x80 != 0;
		let len = match byte {
			0..=0x4b => 1,
			0x4c => self.read_u8() as usize,
			0x4d => self.read_u16() as usize,
			0x4e => self.read_u32() as usize,
			_ => return Err(CodecError::InvalidFormat),
		};

		let bytes = self.read_bytes(len).unwrap();
		if negative {
			// Flip sign bit
			if let Some(byte) = bytes.to_owned().get_mut(len - 1) {
				*byte ^= 0x80;
			} else {
				return Err(CodecError::InvalidFormat)
			}
			// bytes.get_mut()[len - 1] ^= 0x80;
		}
		//TODO:: need to check be or le and sign
		Ok(BigInt::from_bytes_be(Sign::Minus, bytes))
	}

	/// Reads a signed 128-bit integer from the byte slice.
	pub fn read_i128(&mut self) -> i128 {
		let bytes = self.read_bytes(16).unwrap();
		i128::from_ne_bytes(bytes.try_into().unwrap())
	}

	/// Reads an encoded EC point from the byte slice.
	pub fn read_encoded_ec_point(&mut self) -> Result<&'a [u8], &'static str> {
		let byte = self.read_u8();
		match byte {
			0x02 | 0x03 => Ok(self.read_bytes(32).unwrap()),
			_ => Err("Invalid encoded EC point"),
		}
	}

	/// Reads a byte slice of the given length from the byte slice.
	pub fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], CodecError> {
		let start = self.pointer;
		self.pointer += count;
		self.data
			.get(start..self.pointer)
			.ok_or_else(|| CodecError::IndexOutOfBounds("Out of bounds".to_string()))
	}

	/// Reads a variable-length byte slice from the byte slice.
	pub fn read_var_bytes(&mut self) -> Result<&'a [u8], CodecError> {
		let len = self.read_var_int().unwrap() as usize;
		self.read_bytes(len)
	}

	/// Reads a variable-length integer from the byte slice.
	pub fn read_var_int(&mut self) -> Result<i64, CodecError> {
		let first = self.read_u8();
		match first {
			0xfd => Ok(self.read_u16() as i64),
			0xfe => Ok(self.read_u32() as i64),
			0xff => Ok(self.read_u64() as i64),
			_ => Ok(first as i64),
		}
	}

	/// Reads a string from the byte slice.
	pub fn read_string(&mut self) -> Result<String, CodecError> {
		let bytes = self.read_var_bytes().unwrap();

		let string = match String::from_utf8(bytes.to_vec()) {
			Ok(s) => s,
			Err(e) => {
				// Handle invalid UTF-8
				return Err(CodecError::InvalidEncoding(e.to_string()))
			},
		};

		// Trim null bytes from end
		let string = string.trim_end_matches(char::from(0));

		Ok(string.to_string())
	}

	/// Reads a push byte slice from the byte slice.
	pub fn read_push_bytes(&mut self) -> Result<&'a [u8], CodecError> {
		let opcode = self.read_u8();
		let len = match opcode {
			0x01..=0x4B => opcode as usize,
			0x4C => self.read_u8() as usize,
			0x4D => self.read_u16() as usize,
			0x4E => self.read_u32() as usize,
			_ => return Err(CodecError::InvalidOpCode),
		};

		self.read_bytes(len)
	}

	/// Reads a push integer from the byte slice.
	pub fn read_push_int(&mut self) -> Result<i64, CodecError> {
		let opcode = self.read_u8();
		match opcode {
			0x00..=0x16 => Ok(opcode as i64 - 1),
			0x01..=0x04 => {
				let n = match opcode {
					0x51 => 1,
					0x52 => 2,
					0x53 => 4,
					0x54 => 8,
					_ => {
						panic!("Invalid opcode")
					},
				};
				let bytes = self.read_bytes(n).unwrap();
				Ok(i64::from_be_bytes(bytes.try_into().unwrap()))
			},
			_ => Err(CodecError::InvalidOpCode),
		}
	}

	/// Reads a push string from the byte slice.
	pub fn read_push_string(&mut self) -> Result<String, CodecError> {
		let bytes = self.read_push_bytes().unwrap();
		String::from_utf8(Vec::from(bytes))
			.map_err(|_| CodecError::InvalidEncoding("Invalid UTF-8".to_string()))
	}

	// Serialization helper methods

	/// Reads a deserializable value from the byte slice.
	pub fn read_serializable<T: Deserialize<'a>>(&mut self) -> Result<T, CodecError> {
		let value: T = bincode::deserialize(&self.data[self.pointer..])
			.map_err(|_e| CodecError::InvalidFormat)
			.unwrap();
		Ok(value)
	}

	/// Reads a list of deserializable values from the byte slice.
	pub fn read_serializable_list<T: Deserialize<'a>>(&mut self) -> Result<Vec<T>, CodecError> {
		let len = self.read_var_int().unwrap();
		let mut list = Vec::with_capacity(len as usize);
		for _ in 0..len {
			list.push(self.read_serializable().unwrap());
		}
		Ok(list)
	}

	// Other methods like `mark`, `reset`, etc.

	pub fn mark(&mut self) {
		self.marker = self.pointer;
	}

	pub fn reset(&mut self) {
		self.pointer = self.marker;
	}

	// pub fn read_ec_point(&mut self) -> Result<ProjectivePoint, &'static str> {
	// 	let tag = self.read_u8();
	// 	let bytes = match tag {
	// 		0x00 => return Ok(ProjectivePoint::IDENTITY),
	// 		0x02 | 0x03 => self.read_bytes(32),
	// 		0x04 => self.read_bytes(64),
	// 		_ => return Err("Invalid EC point tag"),
	// 	};
	//
	// 	let point = EncodedPoint::from_bytes(bytes).unwrap();
	// 	match ProjectivePoint::from_encoded_point(&point) {
	// 		Some(point) => Ok(point),
	// 		None => Err("Invalid EC point"),
	// 	}
	// }
}
