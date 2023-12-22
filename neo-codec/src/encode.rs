use crate::{CodecError, Decoder, Encoder};
use primitive_types::{H160, H256};
use std::fmt::Debug;

pub trait NeoSerializable {
	type Error: Send + Sync + Debug;

	fn size(&self) -> usize;
	fn encode(&self, writer: &mut Encoder);
	fn decode(reader: &mut Decoder) -> Result<Self, Self::Error>
	where
		Self: Sized;
	fn to_array(&self) -> Vec<u8>;
}

impl NeoSerializable for H160 {
	type Error = CodecError;
	fn size(&self) -> usize {
		H160::len_bytes()
	}
	fn encode(&self, writer: &mut Encoder) {
		writer.write_var_bytes(&self.as_bytes());
	}

	fn decode(reader: &mut Decoder) -> Result<Self, Self::Error>
	where
		Self: Sized,
	{
		reader.read_var_bytes().map(|bytes| H160::from_slice(&bytes))
	}

	fn to_array(&self) -> Vec<u8> {
		self.as_bytes().to_vec()
	}
}

impl NeoSerializable for H256 {
	type Error = CodecError;

	fn size(&self) -> usize {
		H256::len_bytes()
	}
	fn encode(&self, writer: &mut Encoder) {
		writer.write_var_bytes(&self.as_bytes());
	}

	fn decode(reader: &mut Decoder) -> Result<Self, CodecError>
	where
		Self: Sized,
	{
		reader.read_var_bytes().map(|bytes| H256::from_slice(&bytes))
	}

	fn to_array(&self) -> Vec<u8> {
		self.as_bytes().to_vec()
	}
}

pub trait VarSizeTrait {
	fn var_size(&self) -> usize;
}

impl<T: NeoSerializable> VarSizeTrait for Vec<T> {
	fn var_size(&self) -> usize {
		let count_var_size = self.len();
		count_var_size + self.iter().map(|item| item.size()).sum::<usize>()
	}
}

// impl<T:NeoSerializable> VarSizeTrait for &[T] {
// 	fn var_size(&self) -> usize {
// 		let count_var_size = self.len();
// 		count_var_size + self.iter().map(|item| item.size()).sum::<usize>()
// 	}
// }

// fn var_size_of_serializables<T: NeoSerializable>(elements: &[T]) -> usize {
// 	let count_var_size = elements.len();
// 	count_var_size + elements.iter().map(|item| item.size()).sum::<usize>()
// }
