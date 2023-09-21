use crate::{
	constant::NeoConstants,
	neo_error::NeoError,
	protocol::{
		core::{neo_trait::NeoTrait, responses::transaction_attribute::TransactionAttribute},
		http_service::HttpService,
		neo_rust::NeoRust,
	},
	transaction::{
		account_signer::AccountSigner,
		contract_signer::ContractSigner,
		serializable_transaction::SerializableTransaction,
		signer::{Signer, SignerType},
		transaction_error::TransactionError,
		witness::Witness,
	},
	types::{contract_parameter::ContractParameter, Bytes, H160Externsion},
};
use primitive_types::H160;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Getters, Setters, MutGetters, CopyGetters, Default)]
pub struct TransactionBuilder {
	version: u8,
	nonce: u32,
	valid_until_block: Option<u32>,
	signers: Vec<Box<dyn Signer>>,
	additional_network_fee: u64,
	additional_system_fee: u64,
	attributes: Vec<TransactionAttribute>,
	script: Option<Bytes>,
	fee_consumer: Option<Box<dyn Fn(u64, u64)>>,
	fee_error: Option<TransactionError>,
}

impl TransactionBuilder {
	pub const GAS_TOKEN_HASH: H160 =
		H160::from_hex("d2a4cff31913016155e38e474a2c06d08be276cf").unwrap();
	pub const BALANCE_OF_FUNCTION: &'static str = "balanceOf";
	pub const DUMMY_PUB_KEY: &'static str =
		"02ec143f00b88524caf36a0121c2de09eef0519ddbe1c710a00f0e2663201ee4c0";

	// Constructor
	pub fn new() -> Self {
		Self {
			version: 0,
			nonce: 0,
			valid_until_block: None,
			signers: Vec::new(),
			additional_network_fee: 0,
			additional_system_fee: 0,
			attributes: Vec::new(),
			script: None,
			fee_consumer: None,
			fee_error: None,
		}
	}

	// Configuration

	pub fn version(&mut self, version: u8) -> &mut Self {
		self.version = version;
		self
	}

	pub fn nonce(&mut self, nonce: u32) -> Result<&mut Self, TransactionError> {
		// Validate
		if nonce >= u32::MAX {
			return Err(TransactionError::InvalidNonce)
		}

		self.nonce = nonce;
		Ok(self)
	}

	// Other methods

	// Set valid until block
	pub fn valid_until_block(&mut self, block: u32) -> Result<&mut Self, TransactionError> {
		if block == 0 {
			return Err(TransactionError::InvalidBlock)
		}

		self.valid_until_block = Some(block);
		Ok(self)
	}

	// Set script
	pub fn set_script(&mut self, script: Bytes) -> &mut Self {
		self.script = Some(script);
		self
	}

	// Get unsigned transaction
	pub async fn get_unsigned_tx(&mut self) -> Result<SerializableTransaction, TransactionError> {
		// Validate configuration
		if self.signers.is_empty() {
			return Err(TransactionError::NoSigners)
		}

		if self.script.is_none() {
			return Err(TransactionError::NoScript)
		}
		// Validate no duplicate signers
		if self.signers.len() != self.signers.dedup().count() {
			return Err(TransactionError::DuplicateSigner)
		}

		// Check signer limits
		if self.signers.len() > NeoConstants::MAX_SIGNERS {
			return Err(TransactionError::TooManySigners)
		}

		// Validate script
		if let Some(script) = &self.script {
			if script.is_empty() {
				return Err(TransactionError::EmptyScript)
			}
		} else {
			return Err(TransactionError::NoScript)
		}

		// Get fees
		let system_fee = self.get_system_fee().await.unwrap();
		let network_fee = self.get_network_fee().await.unwrap();

		// Check sender balance if needed
		if let Some(fee_consumer) = &self.fee_consumer {
			let sender_balance =
				NeoRust::<HttpService>::instance().get_sender_balance().await.unwrap();
			if network_fee + system_fee > sender_balance {
				fee_consumer(network_fee + system_fee, sender_balance);
			}
		}

		// Build transaction
		Ok(SerializableTransaction::new(
			self.version,
			self.nonce,
			self.valid_until_block.unwrap(),
			self.clone().signers,
			system_fee as i64,
			network_fee as i64,
			self.clone().attributes,
			self.clone().script.unwrap(),
			vec![],
		))
	}

	async fn get_system_fee(&self) -> Result<u64, TransactionError> {
		let script = self.script.as_ref().unwrap();

		let response = NeoRust::<HttpService>::instance()
			.invoke_script(script.to_hex(), vec![self.signers[0]])
			.await
			.request()
			.await
			.unwrap();
		Ok(u64::from_str(response.gas_consumed.as_str()).unwrap()) // example
	}

	async fn get_network_fee(&mut self) -> Result<u64, TransactionError> {
		let unsigned_tx = self.get_unsigned_tx().await.unwrap();

		let fee = NeoRust::<HttpService>::instance().get_network_fee(unsigned_tx).await.unwrap();
		Ok(fee)
	}

	// Get sender balance
	async fn get_sender_balance(&self) -> Result<u64, TransactionError> {
		// Call network
		let sender = &self.signers[0];

		if Self::is_account_signer(sender) {
			let balance = NeoRust::<HttpService>::instance()
				.invoke_function(
					&Self::GAS_TOKEN_HASH,
					Self::BALANCE_OF_FUNCTION.to_string(),
					vec![ContractParameter::hash160(sender.get_signer_hash())],
					vec![],
				)
				.await
				.unwrap();
			return Ok(balance)
		}
		Err(TransactionError::InvalidSender)
	}

	fn is_account_signer(signer: &Box<dyn Signer>) -> bool {
		// let sig = <T as Signer>::SignerType;
		if signer.get_type() == SignerType::Account {
			return true
		}
		return false
	}

	// Sign transaction
	pub async fn sign(&mut self) -> Result<SerializableTransaction, NeoError> {
		let mut transaction = self.get_unsigned_transaction().await.unwrap();

		for signer in &mut transaction.signers {
			if Self::is_account_signer(signer) {
				let account_signer = signer as &mut AccountSigner;
				let acc = &account_signer.account;
				if acc.is_multi_sig() {
					return Err(NeoError::IllegalState(
						"Transactions with multi-sig signers cannot be signed automatically."
							.to_string(),
					))
				}

				let key_pair = acc.key_pair.as_ref().ok_or_else(|| {
                  NeoError::InvalidConfiguration(
                      "Cannot create transaction signature because account does not hold a private key."
                          .to_string(),
                  )
              }).unwrap();

				let tx_bytes = transaction.get_hash_data().await.unwrap();
				transaction.add_witness(Witness::create(tx_bytes, key_pair).unwrap()).unwrap();
			} else {
				let contract_signer = signer as &mut ContractSigner;
				transaction
					.add_witness(
						Witness::create_contract_witness(contract_signer.verify_params.clone())
							.unwrap(),
					)
					.unwrap();
			}
		}

		Ok(transaction)
	}

	// Inside TransactionBuilder impl

	pub async fn get_unsigned_transaction(
		&mut self,
	) -> Result<SerializableTransaction, TransactionError> {
		if self.script.is_none() {
			return Err(TransactionError::TransactionConfiguration(
				"Cannot build a transaction without a script.".to_string(),
			))
		}

		if self.valid_until_block.is_none() {
			let current_block_count = NeoRust::<HttpService>::instance().get_block_count().await;
			self.valid_until_block = Some(
				current_block_count
					+ NeoRust::<HttpService>::instance().max_valid_until_block_increment()
					- 1,
			);
		}

		if self.signers.is_empty() {
			return Err(NeoError::IllegalState(
				"Cannot create a transaction without signers.".to_string(),
			)
			.into())
		}

		if self.is_high_priority() {
			let is_allowed = self.is_allowed_for_high_priority().await.unwrap();
			if !is_allowed {
				return Err(NeoError::IllegalState(
					"Only committee members can send high priority transactions.".to_string(),
				)
				.into())
			}
		}

		let system_fee = self.get_system_fee_for_script().await.unwrap();
		let network_fee = self.calc_network_fee().await.unwrap();
		let fees = system_fee + network_fee;

		if let Some(fee_error) = &self.fee_error {
			if !self.can_send_cover_fees(fees).await.unwrap() {
				return Err(fee_error.clone().into())
			}
		} else if let Some(consumer) = &mut self.fee_consumer {
			let gas_balance = self.get_sender_gas_balance().await.unwrap();
			consumer(fees, gas_balance);
		}

		let transaction = SerializableTransaction::new(
			self.version,
			self.nonce,
			self.valid_until_block.unwrap(),
			self.signers.clone(),
			system_fee,
			network_fee,
			self.attributes.clone(),
			self.script.as_ref().unwrap().clone(),
			vec![],
		);

		Ok(transaction)
	}
}
