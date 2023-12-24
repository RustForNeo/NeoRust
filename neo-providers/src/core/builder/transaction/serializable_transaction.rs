use crate::{
	core::{
		account::AccountTrait,
		transaction::{
			signers::signer::Signer, transaction_attribute::TransactionAttribute,
			transaction_error::TransactionError, witness::Witness,
		},
	},
	JsonRpcClient, Middleware, Provider, ProviderExt,
};
use getset::{Getters, Setters};
use neo_codec::{
	encode::{NeoSerializable, VarSizeTrait},
	Decoder, Encoder,
};
use neo_config::NeoNetwork;
use neo_crypto::hash::HashableForVec;
use neo_types::Bytes;
use serde::Serialize;

#[derive(Debug, Clone, Setters, Getters)]
pub struct SerializableTransaction<T: AccountTrait + Serialize> {
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

impl<T: AccountTrait + Serialize> Eq for SerializableTransaction<T> {}
impl<T: AccountTrait + Serialize> PartialEq for SerializableTransaction<T> {
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

impl<T: AccountTrait + Serialize> SerializableTransaction<T> {
	const HEADER_SIZE: usize = 25;

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
	pub async fn get_hash_data(&self, network: u32) -> Result<Bytes, TransactionError> {
		let mut encoder = Encoder::new();
		self.serialize_without_witnesses(&mut encoder);
		let mut data = encoder.to_bytes().hash256();
		data.splice(0..0, network.to_be_bytes());

		Ok(data)
	}

	fn serialize_without_witnesses(&self, writer: &mut Encoder) {
		writer.write_u8(self.version);
		writer.write_u32(self.nonce);
		writer.write_i64(self.system_fee);
		writer.write_i64(self.network_fee);
		writer.write_u32(self.valid_until_block);
		writer.write_serializable_variable_list(&self.signers);
		writer.write_serializable_variable_list(&self.attributes);
		writer.write_var_bytes(&self.script);
	}
}

impl<T: AccountTrait + Serialize> NeoSerializable for SerializableTransaction<T> {
	type Error = TransactionError;

	fn size(&self) -> usize {
		SerializableTransaction::<T>::HEADER_SIZE
			+ self.signers.var_size()
			+ self.attributes.var_size()
			+ self.script.var_size()
			+ self.witnesses.var_size()
	}

	fn encode(&self, writer: &mut Encoder) {
		self.serialize_without_witnesses(writer);
		writer.write_serializable_variable_list(&self.witnesses);
	}

	fn decode(reader: &mut Decoder) -> Result<Self, Self::Error>
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
		self.encode(&mut writer);
		writer.to_bytes()
	}
}
