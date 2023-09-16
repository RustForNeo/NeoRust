use p256::ecdsa::signature::SignerMut;
use crate::constant::NeoConstants;
use crate::protocol::core::responses::transaction_attribute::TransactionAttribute;
use crate::protocol::neo_rust::NeoRust;
use crate::transaction::account_signer::AccountSigner;
use crate::transaction::contract_signer::ContractSigner;
use crate::transaction::serializable_transaction::SerializableTransaction;
use crate::transaction::signer::Signer;
use crate::transaction::transaction_error::TransactionError;
use crate::transaction::witness::Witness;
use crate::types::Bytes;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionBuilder {
    version: u8,
    nonce: u32,
    valid_until_block: Option<u32>,
    signers: Vec<Signer>,
    additional_network_fee: u64,
    additional_system_fee: u64,
    attributes: Vec<TransactionAttribute>,
    script: Option<Bytes>,
    fee_consumer: Option<Box<dyn Fn(u64, u64)>>,
    fee_error: Option<TransactionError>,
}

impl TransactionBuilder {

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
            return Err(TransactionError::InvalidBlock);
        }

        self.valid_until_block = Some(block);
        Ok(self)
    }

    // Add signer
    pub fn add_signer(&mut self, signer: &Signer) -> Result<&mut Self, TransactionError> {

        // Validate max signers
        if self.signers.len() >= NeoConstants::MAX_SIGNERS {
            return Err(TransactionError::TooManySigners);
        }

        self.signers.push(signer.clone());
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
            return Err(TransactionError::NoSigners);
        }

        if self.script.is_none() {
            return Err(TransactionError::NoScript);
        }
        // Validate no duplicate signers
        if self.signers.len() != self.signers.dedup().count() {
            return Err(TransactionError::DuplicateSigner);
        }

// Check signer limits
        if self.signers.len() > NeoConstants::MAX_SIGNERS {
            return Err(TransactionError::TooManySigners);
        }

// Validate script
        if let Some(script) = &self.script {
            if script.is_empty() {
                return Err(TransactionError::EmptyScript);
            }
        } else {
            return Err(TransactionError::NoScript)
        }


        // Get fees
        let system_fee = self.get_system_fee().await?;
        let network_fee = self.get_network_fee().await?;

        // Check sender balance if needed
        if let Some(fee_consumer) = &self.fee_consumer {
            let sender_balance = NeoRust::instance().get_sender_balance().await?;
            if network_fee + system_fee > sender_balance {
                fee_consumer(network_fee + system_fee, sender_balance);
            }
        }

        // Build transaction
        let tx = SerializableTransaction::new(self.version, self.nonce, self.valid_until_block?, self.clone().signers, system_fee as i64, network_fee as i64, self.clone().attributes, self.clone().script?, vec![]);

        Ok(tx)

    }

    async fn get_system_fee(&self) -> Result<u64, TransactionError> {
        let script = self.script.as_ref().unwrap();

        let response = NeoRust::instance().invoke_script(script).await?;
        Ok(response.gas_consumed.) // example
    }

    async fn get_network_fee(&mut self) -> Result<u64, TransactionError> {
        let unsigned_tx = self.get_unsigned_tx().await?;

        let fee = NeoRust::instance().get_network_fee(unsigned_tx).await?;
        Ok(fee)
    }

    // Get sender balance
    async fn get_sender_balance(&self) -> Result<u64, TransactionError> {
        // Call network
        let sender = &self.signers[0];
        let balance = match sender {
            AccountSigner(account) => NeoRust::instance().get_account_balance(account).await?,
            _ => return Err(TransactionError::InvalidSender),
        };
        Ok(balance)
    }

    // Sign transaction
    pub async fn sign(&mut self) -> Result<SerializableTransaction, TransactionError> {

        let mut tx = self.get_unsigned_tx().await?;

        // Sign all signers
        for signer in &self.signers {
            match signer {
                AccountSigner(account) => {
                    let signature = account.sign(&tx).await?;
                    tx.add_witness(signature);
                },
                ContractSigner(params) => {
                    tx.add_witness(Witness::contract(&params));
                }
            }
        }

        Ok(tx)
    }

}