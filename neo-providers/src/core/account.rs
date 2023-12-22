use crate::core::{transaction::verification_script::VerificationScript, wallet::WalletTrait};
use neo_crypto::{key_pair::KeyPair, keys::Secp256r1PublicKey};
use neo_types::{
	address_or_scripthash::AddressOrScriptHash,
	script_hash::{ScriptHash, ScriptHashExtension},
};
use primitive_types::H160;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{cell::RefCell, fmt::Debug, rc::Weak};

pub trait AccountTrait: Sized + PartialEq + Send + Sync + Debug + Clone {
	type Wallet: WalletTrait;
	type Error: Sync + Send + Debug + Sized;
	type NEP6Account;

	// Methods to access the fields
	fn key_pair(&self) -> &Option<KeyPair>;
	fn address_or_scripthash(&self) -> &AddressOrScriptHash;
	fn label(&self) -> &Option<String>;
	fn verification_script(&self) -> &Option<VerificationScript>;
	fn is_locked(&self) -> bool;
	fn encrypted_private_key(&self) -> &Option<String>;
	fn wallet(&self) -> &Option<Weak<RefCell<Self::Wallet>>>;
	fn signing_threshold(&self) -> &Option<u32>;
	fn nr_of_participants(&self) -> &Option<u32>;
	fn set_key_pair(&mut self, key_pair: Option<KeyPair>);
	fn set_address_or_scripthash(&mut self, address_or_scripthash: AddressOrScriptHash);
	fn set_label(&mut self, label: Option<String>);
	fn set_verification_script(&mut self, verification_script: Option<VerificationScript>);
	fn set_locked(&mut self, is_locked: bool);
	fn set_encrypted_private_key(&mut self, encrypted_private_key: Option<String>);
	fn set_wallet(&mut self, wallet: Option<Weak<RefCell<Self::Wallet>>>);
	fn set_signing_threshold(&mut self, signing_threshold: Option<u32>);
	fn set_nr_of_participants(&mut self, nr_of_participants: Option<u32>);

	fn new(
		address: AddressOrScriptHash,
		label: Option<String>,
		verification_script: Option<VerificationScript>,
		signing_threshold: Option<u32>,
		nr_of_participants: Option<u32>,
	) -> Self;

	fn from_key_pair(
		key_pair: KeyPair,
		signing_threshold: Option<u32>,
		nr_of_participants: Option<u32>,
	) -> Result<Self, Self::Error>;

	fn from_key_pair_opt(
		key_pair: Option<KeyPair>,
		address: AddressOrScriptHash,
		label: Option<String>,
		verification_script: Option<VerificationScript>,
		is_locked: bool,
		encrypted_private_key: Option<String>,
		wallet: Option<Weak<RefCell<Self::Wallet>>>,
		signing_threshold: Option<u32>,
		nr_of_participants: Option<u32>,
	) -> Self;

	fn from_wif(wif: &str) -> Result<Self, Self::Error>;

	fn from_nep6_account(nep6_account: &Self::NEP6Account) -> Result<Self, Self::Error>;

	fn decrypt_private_key(&mut self, password: &str) -> Result<(), Self::Error>;

	fn encrypt_private_key(&mut self, password: &str) -> Result<(), Self::Error>;

	fn get_script_hash(&self) -> ScriptHash;

	fn get_signing_threshold(&self) -> Result<u32, Self::Error>;

	fn get_nr_of_participants(&self) -> Result<u32, Self::Error>;

	fn to_nep6_account(&self) -> Result<Self::NEP6Account, Self::Error>;

	fn from_verification_script(script: &VerificationScript) -> Result<Self, Self::Error>;

	fn from_public_key(public_key: &Secp256r1PublicKey) -> Result<Self, Self::Error>;

	fn create_multi_sig(
		public_keys: &[Secp256r1PublicKey],
		signing_threshold: u32,
	) -> Result<Self, Self::Error>;

	fn from_address(address: &str) -> Result<Self, Self::Error>;

	fn from_script_hash(script_hash: &H160) -> Result<Self, Self::Error>;

	fn create() -> Result<Self, Self::Error>;

	fn is_multi_sig(&self) -> bool;
}
