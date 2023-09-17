use crate::{
	constant::NeoConstants,
	neo_error::NeoError,
	protocol::{core::responses::transaction_attribute::TransactionAttribute, neo_rust::NeoRust},
	transaction::{signer::Signer, transaction_error::TransactionError, witness::Witness},
	types::Bytes,
};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct SerializableTransaction<T>
where
	T: Signer,
{
	version: u8,
	nonce: u32,
	valid_until_block: u32,
	pub(crate) signers: Vec<T>,
	system_fee: i64,
	network_fee: i64,
	attributes: Vec<TransactionAttribute>,
	script: Bytes,
	witnesses: Vec<Witness>,
	block_count_when_sent: Option<u32>,
}

impl<T> SerializableTransaction<T> {
	// Constructor

	pub fn new(
		version: u8,
		nonce: u32,
		valid_until_block: u32,
		signers: Vec<T>,
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
		let hex = hex::encode(self.serialize());

		// Send using NeoRust
		let neo_rust = NeoRust::instance().as_ref().ok_or(NeoError::NeoRustNotInitialized)?;

		neo_rust.send_raw_transaction(hex).await?;

		self.block_count_when_sent = Some(neo_rust.get_block_count().await?);

		Ok(())
	}

	// Get hash data
	pub fn get_hash_data(&self) -> Result<Bytes, TransactionError> {
		let neo_rust = NeoRust::instance().as_ref().ok_or(NeoError::NeoRustNotInitialized)?;

		let network_magic = neo_rust.get_network_magic()?;
		let data = self.serialize_without_witnesses();

		Ok(network_magic + data.sha256())
	}
	// Serialization

	pub async fn serialize(&self) -> Bytes {
		let mut writer = Bytes::new();

		writer.write_u8(self.version).await.expect("Failed to write version");
		writer.write_u32(self.nonce).await.expect("Failed to write nonce");
		writer
			.write_u32(self.valid_until_block)
			.await
			.expect("Failed to write valid_until_block");

		// Write signers
		let signers_len = self.signers.len() as u32;
		writer.write_var_u32(signers_len);
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

		writer
	}

	// Deserialization

	pub fn deserialize(bytes: &[u8]) -> Result<Self, TransactionError> {
		let mut reader = Bytes::from(bytes);

		let version = reader.read_u8()?;
		let nonce = reader.read_u32()?;
		let valid_until_block = reader.read_u32()?;

		// Read signers
		let signers_len = reader.read_var_u32()?;
		let mut signers = Vec::new();
		for _ in 0..signers_len {
			let signer = Signer::deserialize(&mut reader)?;
			signers.push(signer);
		}

		// Read attributes
		let attributes_len = reader.read_var_u32()?;
		let mut attributes = Vec::new();
		for _ in 0..attributes_len {
			let attribute = TransactionAttribute::deserialize(&mut reader)?;
			attributes.push(attribute);
		}

		let script = reader.read_var_bytes()?;

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
