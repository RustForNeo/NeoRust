use crate::{
	private_key_from_wif, public_key_to_address,
	wallet::{
		nep6account::NEP6Account,
		nep6contract::{NEP6Contract, NEP6Parameter},
		wallet::Wallet,
		wallet_error::WalletError,
	},
};
use neo_crypto::{key_pair::KeyPair, nep2::NEP2};
use neo_types::{
	address::Address,
	address_or_scripthash::AddressOrScriptHash,
	contract_parameter_type::ContractParameterType,
	script_hash::{ScriptHash, ScriptHashExtension},
	*,
};

use neo_crypto::keys::Secp256r1PublicKey;
use neo_providers::core::{
	account::AccountTrait, transaction::verification_script::VerificationScript,
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::{
	cell::RefCell,
	hash::{Hash, Hasher},
	rc::Weak,
	str::FromStr,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Account {
	#[serde(skip)]
	pub key_pair: Option<KeyPair>,
	#[serde(
		serialize_with = "serialize_address_or_script_hash",
		deserialize_with = "deserialize_address_or_script_hash"
	)]
	address_or_scripthash: AddressOrScriptHash,
	label: Option<String>,
	pub verification_script: Option<VerificationScript>,
	is_locked: bool,
	encrypted_private_key: Option<String>,
	wallet: Option<Weak<RefCell<Wallet>>>,
	signing_threshold: Option<u32>,
	nr_of_participants: Option<u32>,
}

impl PartialEq for Account {
	fn eq(&self, other: &Self) -> bool {
		self.address_or_scripthash == other.address_or_scripthash
			&& self.label == other.label
			&& self.verification_script == other.verification_script
			&& self.is_locked == other.is_locked
			&& self.encrypted_private_key == other.encrypted_private_key
			&& self.signing_threshold == other.signing_threshold
			&& self.nr_of_participants == other.nr_of_participants
	}
}

impl Hash for Account {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.address_or_scripthash.hash(state);
		self.label.hash(state);
		self.verification_script.hash(state);
		self.is_locked.hash(state);
		self.encrypted_private_key.hash(state);
		self.signing_threshold.hash(state);
		self.nr_of_participants.hash(state);
	}
}

impl AccountTrait for Account {
	type Wallet = Wallet;
	type Error = WalletError;
	type NEP6Account = NEP6Account;

	fn key_pair(&self) -> &Option<KeyPair> {
		&self.key_pair
	}

	fn address_or_scripthash(&self) -> &AddressOrScriptHash {
		&self.address_or_scripthash
	}

	fn label(&self) -> &Option<String> {
		&self.label
	}

	fn verification_script(&self) -> &Option<VerificationScript> {
		&self.verification_script
	}

	fn is_locked(&self) -> bool {
		self.is_locked
	}

	fn encrypted_private_key(&self) -> &Option<String> {
		&self.encrypted_private_key
	}

	fn wallet(&self) -> &Option<Weak<RefCell<Self::Wallet>>> {
		&self.wallet
	}

	fn signing_threshold(&self) -> &Option<u32> {
		&self.signing_threshold
	}

	fn nr_of_participants(&self) -> &Option<u32> {
		&self.nr_of_participants
	}

	fn set_key_pair(&mut self, key_pair: Option<KeyPair>) {
		self.key_pair = key_pair;
	}

	fn set_address_or_scripthash(&mut self, address_or_scripthash: AddressOrScriptHash) {
		self.address_or_scripthash = address_or_scripthash;
	}

	fn set_label(&mut self, label: Option<String>) {
		self.label = label;
	}

	fn set_verification_script(&mut self, verification_script: Option<VerificationScript>) {
		self.verification_script = verification_script;
	}

	fn set_locked(&mut self, is_locked: bool) {
		self.is_locked = is_locked;
	}

	fn set_encrypted_private_key(&mut self, encrypted_private_key: Option<String>) {
		self.encrypted_private_key = encrypted_private_key;
	}

	fn set_wallet(&mut self, wallet: Option<Weak<RefCell<Self::Wallet>>>) {
		self.wallet = wallet;
	}

	fn set_signing_threshold(&mut self, signing_threshold: Option<u32>) {
		self.signing_threshold = signing_threshold;
	}

	fn set_nr_of_participants(&mut self, nr_of_participants: Option<u32>) {
		self.nr_of_participants = nr_of_participants;
	}

	// Constructor
	fn new(
		address: AddressOrScriptHash,
		label: Option<String>,
		verification_script: Option<VerificationScript>,
		signing_threshold: Option<u32>,
		nr_of_participants: Option<u32>,
	) -> Self {
		Self {
			key_pair: None,
			address_or_scripthash: address,
			label,
			verification_script,
			is_locked: false,
			encrypted_private_key: None,
			wallet: None,
			signing_threshold,
			nr_of_participants,
		}
	}

	fn from_key_pair(
		key_pair: KeyPair,
		signing_threshold: Option<u32>,
		nr_of_participants: Option<u32>,
	) -> Result<Self, Self::Error> {
		let address = public_key_to_address(&key_pair.public_key);
		Ok(Self {
			key_pair: Some(key_pair.clone()),
			address_or_scripthash: AddressOrScriptHash::Address(address.clone()),
			label: Some(address),
			verification_script: Some(VerificationScript::from_public_key(
				&key_pair.clone().public_key(),
			)),
			is_locked: false,
			encrypted_private_key: None,
			wallet: None,
			signing_threshold,
			nr_of_participants,
		})
	}

	fn from_key_pair_opt(
		key_pair: Option<KeyPair>,
		address: AddressOrScriptHash,
		label: Option<String>,
		verification_script: Option<VerificationScript>,
		is_locked: bool,
		encrypted_private_key: Option<String>,
		wallet: Option<Weak<RefCell<Wallet>>>,
		signing_threshold: Option<u32>,
		nr_of_participants: Option<u32>,
	) -> Self {
		Self {
			key_pair,
			address_or_scripthash: address,
			label,
			verification_script,
			is_locked,
			encrypted_private_key,
			wallet,
			signing_threshold,
			nr_of_participants,
		}
	}

	fn from_wif(wif: &str) -> Result<Self, Self::Error> {
		let key_pair = KeyPair::from_secret_key(&private_key_from_wif(wif).unwrap());
		Self::from_key_pair(key_pair, None, None)
	}

	fn from_nep6_account(nep6_account: &Self::NEP6Account) -> Result<Self, Self::Error> {
		let (verification_script, signing_threshold, nr_of_participants) =
			match nep6_account.contract {
				Some(ref contract) if contract.script.is_some() => {
					let script = contract.script.clone().unwrap();
					let verification_script = VerificationScript::from(script.as_bytes().to_vec());
					let signing_threshold = if verification_script.is_multi_sig() {
						Some(verification_script.get_signing_threshold().unwrap())
					} else {
						None
					};
					let nr_of_participants = if verification_script.is_multi_sig() {
						Some(verification_script.get_nr_of_accounts().unwrap())
					} else {
						None
					};
					(Some(verification_script), signing_threshold, nr_of_participants)
				},
				_ => (None, None, None),
			};

		Ok(Self {
			address_or_scripthash: AddressOrScriptHash::Address(nep6_account.address.clone()),
			label: nep6_account.label.clone(),
			verification_script,
			is_locked: nep6_account.lock,
			encrypted_private_key: nep6_account.key.clone(),
			signing_threshold: signing_threshold.map(|x| x as u32),
			nr_of_participants: nr_of_participants.map(|x| x as u32),
			..Default::default()
		})
	}

	fn decrypt_private_key(&mut self, password: &str) -> Result<(), Self::Error> {
		if self.key_pair.is_some() {
			return Ok(())
		}

		let encrypted_private_key = self
			.encrypted_private_key
			.as_ref()
			.ok_or(Self::Error::AccountState("No encrypted private key present".to_string()))
			.unwrap();
		let key_pair = NEP2::decrypt(password, encrypted_private_key).unwrap();
		self.key_pair = Some(KeyPair::from_secret_key(&key_pair.private_key().clone()));
		Ok(())
	}

	fn encrypt_private_key(&mut self, password: &str) -> Result<(), Self::Error> {
		let key_pair = self
			.key_pair
			.as_ref()
			.ok_or(Self::Error::AccountState("No decrypted key pair present".to_string()))
			.unwrap();
		let encrypted_private_key = NEP2::encrypt(password, key_pair).unwrap();
		self.encrypted_private_key = Some(encrypted_private_key);
		self.key_pair = None;
		Ok(())
	}

	fn get_script_hash(&self) -> ScriptHash {
		self.address_or_scripthash.script_hash()
	}

	fn get_signing_threshold(&self) -> Result<u32, Self::Error> {
		self.signing_threshold
			.ok_or_else(|| Self::Error::AccountState("Account is not MultiSig".to_string()))
	}

	fn get_nr_of_participants(&self) -> Result<u32, Self::Error> {
		self.nr_of_participants
			.ok_or_else(|| Self::Error::AccountState("Account is not MultiSig".to_string()))
	}

	// pub async fn get_nep17_balances(&self) -> Result<HashMap<H160, u32>, Self::Error> {
	// 	let balances = HTTP_PROVIDER
	// 		.read()
	// 		.unwrap()
	// 		.get_nep17_balances(self.get_script_hash().clone())
	// 		.request()
	// 		.await
	// 		.unwrap();
	// 	let mut nep17_balances = HashMap::new();
	// 	for balance in balances.balances {
	// 		nep17_balances.insert(balance.asset_hash, u32::from_str(&balance.amount).unwrap());
	// 	}
	// 	Ok(nep17_balances)
	// }

	fn to_nep6_account(&self) -> Result<Self::NEP6Account, Self::Error> {
		if self.key_pair.is_some() && self.encrypted_private_key.is_none() {
			return Err(Self::Error::AccountState(
				"Account private key is decrypted but not encrypted".to_string(),
			))
		}

		let contract = match &self.verification_script {
			Some(script) => {
				let parameters = if script.is_multi_sig() {
					let threshold = script.get_signing_threshold().unwrap();
					let nr_accounts = script.get_nr_of_accounts().unwrap();
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
					script: Some(script.script().to_base64()),
					nep6_parameters: parameters,
					is_deployed: false,
				})
			},
			None => None,
		};

		Ok(Self::NEP6Account {
			address: self.address_or_scripthash.address(),
			label: self.label.clone(),
			is_default: false, // TODO
			lock: self.is_locked,
			key: self.encrypted_private_key.clone(),
			contract,
			extra: None,
		})
	}

	// Static methods

	fn from_verification_script(script: &VerificationScript) -> Result<Self, Self::Error> {
		let address = ScriptHash::from_script(&script.script());

		let (signing_threshold, nr_of_participants) = if script.is_multi_sig() {
			(
				Some(script.get_signing_threshold().unwrap()),
				Some(script.get_nr_of_accounts().unwrap()),
			)
		} else {
			(None, None)
		};

		Ok(Self {
			address_or_scripthash: AddressOrScriptHash::ScriptHash(address),
			label: Some(ScriptHashExtension::to_string(&address)),
			verification_script: Some(script.clone()),
			signing_threshold: signing_threshold.map(|x| x as u32),
			nr_of_participants: nr_of_participants.map(|x| x as u32),
			..Default::default()
		})
	}

	fn from_public_key(public_key: &Secp256r1PublicKey) -> Result<Self, Self::Error> {
		let script = VerificationScript::from_public_key(public_key);
		let address = ScriptHash::from_script(&script.script());

		Ok(Self {
			address_or_scripthash: AddressOrScriptHash::ScriptHash(address),
			label: Some(ScriptHashExtension::to_string(&address)),
			verification_script: Some(script),
			..Default::default()
		})
	}

	fn create_multi_sig(
		public_keys: &[Secp256r1PublicKey],
		signing_threshold: u32,
	) -> Result<Self, Self::Error> {
		let script = VerificationScript::from_multi_sig(public_keys, signing_threshold as u8);

		Ok(Self {
			label: Some(script.script().to_base64()),
			verification_script: Some(script),
			signing_threshold: Some(signing_threshold),
			nr_of_participants: Some(public_keys.len() as u32),
			..Default::default()
		})
	}

	fn from_address(address: &str) -> Result<Self, Self::Error> {
		let address = Address::from_str(address).unwrap();
		Ok(Self {
			address_or_scripthash: AddressOrScriptHash::Address(address.clone()),
			label: Some(address),
			..Default::default()
		})
	}

	fn from_script_hash(script_hash: &H160) -> Result<Self, Self::Error> {
		let address = script_hash.to_address();
		Self::from_address(&address)
	}

	fn create() -> Result<Self, Self::Error> {
		let key_pair = KeyPair::new_random();
		Self::from_key_pair(key_pair, None, None)
	}

	fn is_multi_sig(&self) -> bool {
		self.signing_threshold.is_some() && self.nr_of_participants.is_some()
	}
}
