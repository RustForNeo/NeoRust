use crate::protocol::core::neo_trait::NeoTrait;
use crate::types::PrivateKey;
use crate::{
	crypto::{key_pair::KeyPair, nep2::NEP2},
	protocol::neo_rust::NeoRust,
	script::verification_script::VerificationScript,
	types::{contract_parameter_type::ContractParameterType, Address, H160Externsion},
	wallet::{
		nep6account::NEP6Account,
		nep6contract::{NEP6Contract, NEP6Parameter},
		wallet::Wallet,
		wallet_error::WalletError,
	},
};
use p256::PublicKey;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Account {
	pub(crate) key_pair: Option<KeyPair>,
	address: Address,
	label: Option<String>,
	verification_script: Option<VerificationScript>,
	is_locked: bool,
	encrypted_private_key: Option<String>,
	wallet: Option<Wallet>,
	signing_threshold: Option<i32>,
	nr_of_participants: Option<i32>,
}

impl Account {
	// Constructors

	pub fn new(
		address: Address,
		label: Option<String>,
		verification_script: Option<VerificationScript>,
		signing_threshold: Option<i32>,
		nr_of_participants: Option<i32>,
	) -> Self {
		Self {
			key_pair: None,
			address,
			label,
			verification_script,
			is_locked: false,
			encrypted_private_key: None,
			wallet: None,
			signing_threshold,
			nr_of_participants,
		}
	}

	pub fn from_key_pair(
		key_pair: KeyPair,
		signing_threshold: Option<i32>,
		nr_of_participants: Option<i32>,
	) -> Result<Self, WalletError> {
		let address = key_pair.get_address()?;
		Ok(Self {
			key_pair: Some(key_pair),
			address,
			label: Some(address.to_string()),
			verification_script: Some(VerificationScript::from_public_key(&key_pair.public_key)?),
			is_locked: false,
			encrypted_private_key: None,
			wallet: None,
			signing_threshold,
			nr_of_participants,
		})
	}

	pub fn from_key_pair_opt(
		key_pair: Option<KeyPair>,
		address: Address,
		label: Option<String>,
		verification_script: Option<VerificationScript>,
		is_locked: bool,
		encrypted_private_key: Option<String>,
		wallet: Option<Wallet>,
		signing_threshold: Option<i32>,
		nr_of_participants: Option<i32>,
	) -> Self {
		Self {
			key_pair,
			address,
			label,
			verification_script,
			is_locked,
			encrypted_private_key,
			wallet,
			signing_threshold,
			nr_of_participants,
		}
	}

	pub fn from_wif(wif: &str) -> Result<Self, WalletError> {
		let private_key = PrivateKey::from_private_key_wif(wif)?;
		let key_pair = KeyPair::from_private_key(private_key)?;
		Self::from_key_pair(key_pair, None, None)
	}

	pub fn from_nep6_account(nep6_account: &NEP6Account) -> Result<Self, WalletError> {
		let (verification_script, signing_threshold, nr_of_participants) =
			match nep6_account.contract {
				Some(ref contract) if !contract.script.is_empty() => {
					let script = contract.script.as_bytes();
					let verification_script = VerificationScript::from_bytes(script)?;
					let signing_threshold = if verification_script.is_multisig() {
						Some(verification_script.get_signing_threshold()?)
					} else {
						None
					};
					let nr_of_participants = if verification_script.is_multisig() {
						Some(verification_script.get_nr_of_accounts()?)
					} else {
						None
					};
					(Some(verification_script), signing_threshold, nr_of_participants)
				},
				_ => (None, None, None),
			};

		Ok(Self {
			address: nep6_account.address.clone(),
			label: nep6_account.label.clone(),
			verification_script,
			is_locked: nep6_account.lock,
			encrypted_private_key: nep6_account.key.clone(),
			signing_threshold,
			nr_of_participants,
			..Default::default()
		})
	}

	// Instance methods

	pub fn label(&mut self, label: &str) {
		self.label = Some(label.to_string());
	}

	pub fn wallet(&mut self, wallet: Option<Wallet>) {
		self.wallet = wallet;
	}

	pub fn lock(&mut self) {
		self.is_locked = true;
	}

	pub fn unlock(&mut self) {
		self.is_locked = false;
	}

	pub fn decrypt_private_key(&mut self, password: &str) -> Result<(), WalletError> {
		if self.key_pair.is_some() {
			return Ok(());
		}

		let encrypted_private_key = self
			.encrypted_private_key
			.as_ref()
			.ok_or(WalletError::AccountState("No encrypted private key present".to_string()))?;
		let key_pair = NEP2::decrypt(password, encrypted_private_key)?;
		self.key_pair = Some(KeyPair::from_private_key(&key_pair));
		Ok(())
	}

	pub fn encrypt_private_key(&mut self, password: &str) -> Result<(), WalletError> {
		let key_pair = self
			.key_pair
			.as_ref()
			.ok_or(WalletError::AccountState("No decrypted key pair present".to_string()))?;
		let encrypted_private_key = NEP2::encrypt(password, &key_pair.private_key)?;
		self.encrypted_private_key = Some(encrypted_private_key);
		self.key_pair = None;
		Ok(())
	}

	pub fn get_script_hash(&self) -> Result<H160, WalletError> {
		H160::from_address(&self.address)
	}

	pub fn get_signing_threshold(&self) -> Result<i32, WalletError> {
		self.signing_threshold
			.ok_or_else(|| WalletError::AccountState("Account is not multisig".to_string()))
	}

	pub fn get_nr_of_participants(&self) -> Result<i32, WalletError> {
		self.nr_of_participants
			.ok_or_else(|| WalletError::AccountState("Account is not multisig".to_string()))
	}

	pub async fn get_nep17_balances(&self) -> Result<HashMap<H160, i32>, WalletError> {
		let balances = NeoRust::instance().get_nep17_balances(self.get_script_hash()?).await;
		let mut nep17_balances = HashMap::new();
		for balance in balances {
			nep17_balances.insert(balance.asset_hash, balance.amount.to_i32()?);
		}
		Ok(nep17_balances)
	}

	pub fn to_nep6_account(&self) -> Result<NEP6Account, WalletError> {
		if self.key_pair.is_some() && self.encrypted_private_key.is_none() {
			return Err(WalletError::AccountState(
				"Account private key is decrypted but not encrypted".to_string(),
			));
		}

		let contract = match &self.verification_script {
			Some(script) => {
				let parameters = if script.is_multisig() {
					let threshold = script.get_signing_threshold()?;
					let nr_accounts = script.get_nr_of_accounts()?;
					(0..nr_accounts)
						.map(|i| NEP6Parameter {
							param_name: format!("signature{}", i),
							param_type: ContractParameterType::Signature,
						})
						.collect()
				} else if script.is_single_sig() {
					vec![NEP6Parameter {
						param_name: "signature".to_string(),
						param_type: ContractParameterType::Signature,
					}]
				} else {
					vec![]
				};

				Some(NEP6Contract {
					script: script.to_bytes().to_base64(),
					nep6_parameters: parameters,
					is_deployed: false,
				})
			},
			None => None,
		};

		Ok(NEP6Account {
			address: self.address.clone(),
			label: self.label.clone(),
			is_default: false, // TODO
			lock: self.is_locked,
			key: self.encrypted_private_key.clone(),
			contract,
			extra: None,
		})
	}

	// Static methods

	pub fn from_verification_script(script: &VerificationScript) -> Result<Self, WalletError> {
		let address = H160::from_script(&script.to_bytes()?).to_address();

		let (signing_threshold, nr_of_participants) = if script.is_multisig() {
			(Some(script.get_signing_threshold()?), Some(script.get_nr_of_accounts()?))
		} else {
			(None, None)
		};

		Ok(Self {
			address,
			label: Some(address.to_string()),
			verification_script: Some(script.clone()),
			signing_threshold,
			nr_of_participants,
			..Default::default()
		})
	}

	pub fn from_public_key(public_key: &PublicKey) -> Result<Self, WalletError> {
		let script = VerificationScript::from_public_key(public_key)?;
		let address = H160::from_script(&script.to_bytes()?).to_address();

		Ok(Self {
			address,
			label: Some(address.to_string()),
			verification_script: Some(script),
			..Default::default()
		})
	}

	pub fn create_multisig(
		public_keys: &[PublicKey],
		signing_threshold: i32,
	) -> Result<Self, WalletError> {
		let script = VerificationScript::multisig(public_keys, signing_threshold)?;
		let address = H160::from_script(&script.to_bytes()?).to_address();

		Ok(Self {
			address,
			label: Some(address.to_string()),
			verification_script: Some(script),
			signing_threshold: Some(signing_threshold),
			nr_of_participants: Some(public_keys.len() as i32),
			..Default::default()
		})
	}

	pub fn from_address(address: &str) -> Result<Self, WalletError> {
		let address = Address::from_str(address)?;
		Ok(Self { address, label: Some(address.to_string()), ..Default::default() })
	}

	pub fn from_script_hash(script_hash: &H160) -> Result<Self, WalletError> {
		let address = script_hash.to_address();
		Self::from_address(&address)
	}

	pub fn create() -> Result<Self, WalletError> {
		let key_pair = KeyPair::create()?;
		Self::from_key_pair(key_pair, None, None)
	}
}
