use crate::core::{
	account::AccountTrait,
	transaction::{
		signers::signer::Signer, transaction_attribute::TransactionAttribute,
		transaction_error::TransactionError, witness::Witness,
	},
};
use getset::{Getters, Setters};
use neo_codec::{serializable::NeoSerializable, Decoder, Encoder};
use neo_types::{nef_file::HEADER_SIZE, Bytes};
use serde::{de::DeserializeOwned, Serialize};
use std::hash::Hash;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone, Setters, Getters)]
pub struct SerializableTransaction<T: AccountTrait + Serialize + DeserializeOwned> {
	version: u8,
	nonce: u32,
	valid_until_block: u32,
	#[getset(get = "pub")]
	pub(crate) signers: Vec<Signer<T>>,
	#[getset(get = "pub", set = "pub")]
	system_fee: i64,
	#[getset(get = "pub", set = "pub")]
	network_fee: i64,
	attributes: Vec<TransactionAttribute>,
	script: Bytes,
	witnesses: Vec<Witness>,
	block_count_when_sent: Option<u32>,
}

impl<T: AccountTrait + Serialize + DeserializeOwned> Eq for SerializableTransaction<T> {}
impl<T: AccountTrait + Serialize + DeserializeOwned> PartialEq for SerializableTransaction<T> {
	fn eq(&self, other: &Self) -> bool {
		self.version == other.version
			&& self.nonce == other.nonce
			&& self.valid_until_block == other.valid_until_block
			&& self.signers == other.signers
			&& self.system_fee == other.system_fee
			&& self.network_fee == other.network_fee
			&& self.attributes == other.attributes
			&& self.script == other.script
			&& self.witnesses == other.witnesses
	}
}

impl<T: AccountTrait + Serialize + DeserializeOwned> SerializableTransaction<T> {
	// Constructor
	pub fn new(
		version: u8,
		nonce: u32,
		valid_until_block: u32,
		signers: Vec<Signer<T>>,
		system_fee: i64,
		network_fee: i64,
		attributes: Vec<TransactionAttribute>,
		script: Bytes,
		witnesses: Vec<Witness>,
	) -> Self {
		Self {
			version,
			nonce,
			valid_until_block,
			signers,
			system_fee,
			network_fee,
			attributes,
			script,
			witnesses,
			block_count_when_sent: None,
		}
	}

	// Methods
	pub fn add_witness(&mut self, witness: Witness) {
		self.witnesses.push(witness);
	}

	// Send transaction
	// pub async fn send(&mut self) -> Result<(), TransactionError> {
	// 	// Validate transaction
	// 	if self.signers.len() != self.witnesses.len() {
	// 		return Err(TransactionError::InvalidTransaction)
	// 	}
	//
	// 	if self.size() > NeoConstants::MAX_TRANSACTION_SIZE as usize {
	// 		return Err(TransactionError::TxTooLarge)
	// 	}
	//
	// 	// Get hex encoding
	// 	let hex = hex::encode(self.serialize());
	//
	// 	NEO_INSTANCE.read().unwrap().send_raw_transaction(hex).request().await.ok();
	//
	// 	self.block_count_when_sent =
	// 		Some(NEO_INSTANCE.read().unwrap().get_block_count().request().await.unwrap());
	//
	// 	Ok(())
	// }

	// Get hash data
	// pub async fn get_hash_data(&self) -> Result<Bytes, TransactionError> {
	// 	let network_magic = NEO_INSTANCE
	// 		.write()
	// 		.unwrap()
	// 		.get_network_magic_number()
	// 		.await
	// 		.unwrap()
	// 		.to_le_bytes();
	// 	let mut data = self.serialize_without_witnesses().hash256();
	//
	// 	data.splice(0..0, network_magic.iter().cloned());
	//
	// 	Ok(data)
	// }

	// Serialization
	pub fn serialize(&self) -> Bytes {
		let mut writer = Encoder::new();

		writer.write_u8(self.version);
		writer.write_u32(self.nonce);
		writer.write_u32(self.valid_until_block);

		// Write signers
		let signers_len = self.signers.len() as u32;
		writer.write_var_int(signers_len as i64);
		for signer in &self.signers {
			// bincode::serialize(signer)
			// signer.serialize(&mut writer).expect("Failed to serialize signer");
			writer.write_serializable(signer);
		}

		// Write attributes
		let attributes_len = self.attributes.len() as u32;
		writer.write_var_int(attributes_len as i64);
		for attribute in &self.attributes {
			// attribute.serialize(&mut writer).expect("Failed to serialize attribute");
			writer.write_serializable(attribute);
		}

		writer.write_var_bytes(&self.script);

		writer.to_bytes()
	}

	// Deserialization

	pub fn deserialize(bytes: &[u8]) -> Result<Self, TransactionError> {
		let mut reader = Decoder::new(bytes);

		let version = reader.read_u8();
		let nonce = reader.read_u32();
		let valid_until_block = reader.read_u32();

		// Read signers
		let signers_len = reader.read_var_int().unwrap() as u32;
		// let mut signers = Vec::new();
		// for _ in 0..signers_len {

		let signers: Vec<Signer<T>> = reader.read_serializable_list::<Signer<T>>().unwrap();
		// signers.push();
		// }

		// Read attributes
		let attributes_len = reader.read_var_int().unwrap();
		let mut attributes = Vec::new();
		// for _ in 0..attributes_len {
		// 	let attr:TransactionAttribute =  bincode::deserialize(&mut reader.);
		//
		// 	// let attribute = TransactionAttribute::deserialize(&mut reader).unwrap();
		// 	attributes.push(attr);
		// }
		// let list:Vec<TransactionAttribute> =  reader.read_serializable_list();

		let script = reader.read_var_bytes().unwrap().to_vec();

		Ok(Self {
			version,
			nonce,
			valid_until_block,
			signers,
			system_fee: 0,
			network_fee: 0,
			attributes,
			script,
			witnesses: vec![],
			block_count_when_sent: None,
		})
	}

	pub fn size(&self) -> usize {
		let mut size = HEADER_SIZE;

		HEADER_SIZE
			+ self.signers.varSize
			+ self.attributes.varSize
			+ self.script.varSize
			+ self.witnesses.varSize
	}

	fn serialize_without_witnesses(&self, writer: &mut Encoder) {
		writer.write_u8(self.version);
		writer.write_u32(self.nonce);
		writer.write_i64(self.system_fee);
		writer.write_i64(self.network_fee);
		writer.write_u32(self.valid_until_block);
		writer.write_serializable_list(&self.signers);
		writer.write_serializable_list(&self.attributes);
		writer.write_var_bytes(&self.script);
	}
}

impl<T: AccountTrait> NeoSerializable for SerializableTransaction<T> {
	type Error = TransactionError;

	fn size(&self) -> usize {
		let mut size = HEADER_SIZE;

		HEADER_SIZE
			+ self.signers.varSize
			+ self.attributes.varSize
			+ self.script.varSize
			+ self.witnesses.varSize
	}

	fn serialize(&self, writer: &mut Encoder) {
		self.serialize_without_witnesses(writer);
		writer.write_serializable_list(&self.witnesses);
	}

	fn deserialize(reader: &mut Decoder) -> Result<Self, Self::Error>
	where
		Self: Sized,
	{
		let version = reader.read_u8();
		let nonce = reader.read_u32();
		let system_fee = reader.read_i64();
		let network_fee = reader.read_i64();
		let valid_until_block = reader.read_u32();

		// Read signers
		let signers: Vec<Signer<T>> = reader.read_serializable_list::<Signer<T>>().unwrap();

		// Read attributes
		let attributes: Vec<TransactionAttribute> =
			reader.read_serializable_list::<TransactionAttribute>().unwrap();

		let script = reader.read_var_bytes().unwrap().to_vec();

		let mut witnesses = vec![];
		if (reader.available() > 0) {
			witnesses.append(&mut reader.read_serializable_list::<Witness>().unwrap());
		}

		Ok(Self {
			version,
			nonce,
			valid_until_block,
			signers,
			system_fee,
			network_fee,
			attributes,
			script,
			witnesses,
			block_count_when_sent: None,
		})
	}

	fn to_array(&self) -> Vec<u8> {
		let mut writer = Encoder::new();
		self.serialize(&mut writer);
		writer.to_array()
	}
}
