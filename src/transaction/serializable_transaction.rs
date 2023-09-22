use crate::{
	constant::NeoConstants,
	neo_error::NeoError,
	protocol::{
		core::{neo_trait::NeoTrait, responses::transaction_attribute::TransactionAttribute},
		http_service::HttpService,
		neo_rust::NeoRust,
	},
	serialization::{binary_reader::BinaryReader, binary_writer::BinaryWriter},
	transaction::{signer::Signer, transaction_error::TransactionError, witness::Witness},
	types::Bytes,
};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SerializableTransaction {
	version: u8,
	nonce: u32,
	valid_until_block: u32,
	pub(crate) signers: Vec<Signer>,
	system_fee: i64,
	network_fee: i64,
	attributes: Vec<TransactionAttribute>,
	script: Bytes,
	witnesses: Vec<Witness>,
	block_count_when_sent: Option<u32>,
}

impl SerializableTransaction {
	// Constructor

	pub fn new(
		version: u8,
		nonce: u32,
		valid_until_block: u32,
		signers: Vec<Signer>,
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
	pub async fn send(&mut self) -> Result<(), TransactionError> {
		// Validate transaction
		if self.signers.len() != self.witnesses.len() {
			return Err(TransactionError::InvalidTransaction)
		}

		if self.size() > NeoConstants::MAX_TRANSACTION_SIZE {
			return Err(TransactionError::TxTooLarge)
		}

		// Get hex encoding
		let hex = hex::encode(self.serialize().await);

		NeoRust::instance().send_raw_transaction(hex).request().await.ok();

		self.block_count_when_sent =
			Some(NeoRust::instance().get_block_count().request().await.unwrap());

		Ok(())
	}

	// Get hash data
	pub async fn get_hash_data(&self) -> Result<Bytes, TransactionError> {
		let network_magic = NeoRust::instance().get_network_magic_number().await.unwrap();
		let data = self.serialize_without_witnesses();

		Ok(network_magic + data.sha256())
	}
	// Serialization

	pub async fn serialize(&self) -> Bytes {
		let mut writer = BinaryWriter::new();

		writer.write_u8(self.version);
		writer.write_u32(self.nonce);
		writer.write_u32(self.valid_until_block);

		// Write signers
		let signers_len = self.signers.len() as u32;
		writer.write_var_int(signers_len as i64);
		for signer in &self.signers {
			signer.serialize(&mut writer).expect("Failed to serialize signer");
		}

		// Write attributes
		let attributes_len = self.attributes.len() as u32;
		writer.write_var_u32(attributes_len);
		for attribute in &self.attributes {
			attribute.serialize(&mut writer).expect("Failed to serialize attribute");
		}

		writer.write_var_bytes(&self.script);

		writer.to_bytes()
	}

	// Deserialization

	pub fn deserialize(bytes: &[u8]) -> Result<Self, TransactionError> {
		let mut reader = BinaryReader::new(bytes);

		let version = reader.read_u8();
		let nonce = reader.read_u32();
		let valid_until_block = reader.read_u32();

		// Read signers
		let signers_len = reader.read_var_u32().unwrap();
		let mut signers = Vec::new();
		for _ in 0..signers_len {
			let signer = Signer::deserialize(&mut reader).unwrap();
			signers.push(signer);
		}

		// Read attributes
		let attributes_len = reader.read_var_u32().unwrap();
		let mut attributes = Vec::new();
		for _ in 0..attributes_len {
			let attribute = TransactionAttribute::deserialize(&mut reader).unwrap();
			attributes.push(attribute);
		}

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
}
