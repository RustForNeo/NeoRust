use num_bigint::BigInt;

pub trait ToBytesPadded {
	fn to_bytes_padded(&self, length: usize) -> Vec<u8>;
}

impl ToBytesPadded for BigInt {
	fn to_bytes_padded(&self, length: usize) -> Vec<u8> {
		let mut bytes = self.to_signed_bytes_be();
		if bytes.len() < length {
			let mut padded = vec![0u8; length];
			padded[length - bytes.len()..].copy_from_slice(&bytes);
			padded
		} else {
			bytes
		}
	}
}

fn power_of(base: i32, exp: i32) -> i32 {
	base.pow(exp as u32)
}

fn var_size(n: i128) -> usize {
	match n {
		n if n < 0xfd => 1,
		n if n <= 0xffff => 3,
		n if n <= 0xffffffff => 5,
		_ => 9,
	}
}

fn to_unsigned(n: i32) -> u32 {
	n as u32
}

trait ToBytes {
	fn to_bytes(&self) -> Vec<u8>;
}

impl ToBytes for i32 {
	fn to_bytes(&self) -> Vec<u8> {
		self.to_be_bytes().to_vec()
	}
}

impl ToBytes for i64 {
	fn to_bytes(&self) -> Vec<u8> {
		self.to_be_bytes().to_vec()
	}
}

impl ToBytes for f32 {
	fn to_bytes(&self) -> Vec<u8> {
		self.to_be_bytes().to_vec()
	}
}

impl ToBytes for f64 {
	fn to_bytes(&self) -> Vec<u8> {
		self.to_be_bytes().to_vec()
	}
}

fn to_milliseconds(datetime: chrono::DateTime<chrono::Utc>) -> i64 {
	datetime.timestamp_millis()
}
