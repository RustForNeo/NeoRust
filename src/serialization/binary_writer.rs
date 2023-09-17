use serde::Serialize;
use std::hash::Hasher;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryWriter {
	data: Vec<u8>,
}

impl BinaryWriter {
	pub fn new() -> Self {
		Self { data: Vec::new() }
	}

	pub fn write_u8(&mut self, value: u8) {
		self.data.push(value);
	}

	pub fn write_i16(&mut self, v: i16) {
		self.write_u16(v as u16);
	}

	pub fn write_i32(&mut self, v: i32) {
		self.write_u32(v as u32);
	}

	pub fn write_i64(&mut self, v: i64) {
		self.write_u64(v as u64);
	}

	pub fn write_u16(&mut self, v: u16) {
		self.data.extend_from_slice(&v.to_be_bytes());
	}

	pub fn write_u32(&mut self, v: u32) {
		self.data.extend_from_slice(&v.to_be_bytes());
	}

	pub fn write_bytes(&mut self, bytes: &[u8]) {
		self.data.extend_from_slice(bytes);
	}

	// Other primitive write methods
	pub fn write_var_int(&mut self, value: i64) {
		match value {
			0..=0xfd => self.write_u8(value as u8),
			0x10000..=0xffffffff => {
				self.write_u8(0xfd);
				self.write_u16(value as u16);
			},
			_ => {
				self.write_u8(0xff);
				self.write_u64(value as u64);
			},
		}
	}

	pub fn write_string(&mut self, v: &str) {
		self.write_bytes(v.as_bytes());
	}

	pub fn write_fixed_string(&mut self, v: &Option<String>, length: usize) -> std::io::Result<()> {
		let bytes = v.as_deref().unwrap_or_default().as_bytes();
		if bytes.len() > length {
			return Err(std::io::Error::new(
				std::io::ErrorKind::InvalidInput,
				"String longer than specified length",
			))
		}
		let mut padded = vec![0; length];
		padded[0..bytes.len()].copy_from_slice(bytes);
		Ok(self.write_bytes(&padded))
	}

	pub fn write_var_bytes(&mut self, bytes: &[u8]) {
		self.write_var_int(bytes.len() as i64);
		self.write_bytes(bytes);
	}

	// Serialization helpers

	pub fn write_serializable<S: Serialize>(&mut self, value: &S) {
		value.serialize(self).expect("Failed to serialize value");
	}

	pub fn write_serializable_list<S: Serialize>(&mut self, values: &[S]) {
		self.write_var_int(values.len() as i64);
		for item in values {
			self.write_serializable(item);
		}
	}

	pub fn reset(&mut self) {
		self.data.clear();
	}

	pub fn to_bytes(&self) -> Vec<u8> {
		self.data.clone()
	}
}
