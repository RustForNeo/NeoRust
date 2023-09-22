use crate::{
	crypto::{hash::HashableForVec, wif::Wif},
	neo_error::{NeoError, NeoError::InvalidPublicKey},
	protocol::core::responses::{
		transaction_attribute::TransactionAttribute, transaction_send_token::TransactionSendToken,
	},
	utils::*,
};
use base64::{engine::general_purpose, Engine};
use crypto::{ripemd160::Ripemd160, scrypt::ScryptParams, sha2::Sha256};
use futures::TryFutureExt;
use hex::FromHexError;
use p256::{
	ecdsa::{SigningKey, VerifyingKey},
	elliptic_curve::{
		group::prime::PrimeCurveAffine,
		sec1::{FromEncodedPoint, ToEncodedPoint},
	},
	pkcs8::der::{Decode, Encode},
};
use primitive_types::{H160, H256};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Digest;

pub mod call_flags;
pub mod contract_parameter;
pub mod contract_parameter_type;
pub mod plugin_type;
pub mod vm_state;

// Bring EC types into scope

pub type PrivateKey = SigningKey;

pub type PublicKey = VerifyingKey;

pub type Address = H160;

pub type Byte = u8;
pub type Bytes = Vec<u8>;

pub trait H160Externsion
where
	Self: Sized,
{
	fn to_string(&self) -> String;

	fn from_slice(slice: &[u8]) -> Result<Self, NeoError>;

	fn from_hex(hex: &str) -> Result<Self, hex::FromHexError>;
	fn from_address(address: &str) -> Result<Self, NeoError>;

	fn from_public_key(public_key: &PublicKey) -> Self;
	fn to_address(&self) -> String;
	fn to_vec(&self) -> Vec<u8>;
	fn from_script(script: &[u8]) -> Self;
}

impl H160Externsion for H160 {
	fn to_string(&self) -> String {
		bs58::encode(self.0).into_string()
	}

	fn from_slice(slice: &[u8]) -> Result<Self, NeoError> {
		if slice.len() != 20 {
			return Err(NeoError::InvalidAddress)
		}

		let mut arr = [0u8; 20];
		arr.copy_from_slice(slice);
		Ok(Self(arr))
	}

	fn from_hex(hex: &str) -> Result<Self, FromHexError> {
		let bytes = hex::decode(hex).unwrap();
		Ok(Self::from_slice(&bytes))
	}

	fn from_address(address: &str) -> Result<Self, NeoError> {
		let bytes = bs58::decode(address).into_vec().map_err(|_| NeoError::InvalidScript).unwrap();

		Ok(Self::from_slice(&bytes))
	}

	fn from_public_key(public_key: &PublicKey) -> Self {
		let hash = public_key.to_encoded_point(false).as_bytes().sha256_ripemd160();

		let mut arr = [0u8; 20];
		arr.copy_from_slice(&hash);
		Self(arr)
	}

	fn to_address(&self) -> String {
		bs58::encode(&self.0).into_string()
	}

	fn to_vec(&self) -> Vec<u8> {
		self.0.to_vec().unwrap()
	}

	fn from_script(script: &[u8]) -> Self {
		let mut hasher = Sha256::new();
		hasher.update(script);
		let hash = hasher.finalize();

		let mut ripemd = Ripemd160::new();
		ripemd.update(&hash);
		let result = ripemd.finalize();
		let mut arr = [0u8; 20];
		arr.copy_from_slice(&result.into_bytes());
		Self(arr)
	}
}

pub trait PublicKeyExtension
where
	Self: Sized,
{
	fn to_address(&self) -> String;
	fn to_vec(&self) -> Vec<u8>;

	fn from_slice(slice: &[u8]) -> Result<Self, NeoError>;
	fn from_hex(hex: &str) -> Result<Self, hex::FromHexError>;
	fn from_private_key(private_key: &PrivateKey) -> Self;
}

pub trait PrivateKeyExtension
where
	Self: Sized,
{
	fn to_address(&self) -> String;
	fn to_vec(&self) -> Vec<u8>;

	fn to_wif(&self) -> String;

	fn from_slice(slice: &[u8]) -> Result<Self, NeoError>;
	fn from_hex(hex: &str) -> Result<Self, hex::FromHexError>;
}

impl PublicKeyExtension for PublicKey {
	fn to_address(&self) -> String {
		H160::from_public_key(self).to_address()
	}

	fn to_vec(&self) -> Vec<u8> {
		self.as_bytes().to_vec()
	}

	fn from_slice(slice: &[u8]) -> Result<Self, NeoError> {
		if slice.len() != 64 {
			return Err(InvalidPublicKey)
		}

		let mut arr = [0u8; 64];
		arr.copy_from_slice(slice);
		Ok(Self::from_encoded_point(&arr).map_err(|_| InvalidPublicKey).unwrap())
	}

	fn from_hex(hex: &str) -> Result<Self, FromHexError> {
		let bytes = hex::decode(hex).unwrap();
		Ok(Self::from_slice(&bytes).unwrap())
	}

	fn from_private_key(private_key: &PrivateKey) -> Self {
		private_key.public_key()
	}
}

impl PrivateKeyExtension for PrivateKey {
	fn to_address(&self) -> String {
		self.public_key().to_address()
	}

	fn to_vec(&self) -> Vec<u8> {
		self.to_bytes().to_vec()
	}

	fn to_wif(&self) -> String {
		self.to_vec().as_slice().to_wif()
	}

	fn from_slice(slice: &[u8]) -> Result<Self, NeoError> {
		if slice.len() != 32 {
			return Err(InvalidPublicKey)
		}

		let mut arr = [0u8; 32];
		arr.copy_from_slice(slice);
		Ok(Self::from_bytes(&arr).map_err(|_| InvalidPublicKey).unwrap())
	}

	fn from_hex(hex: &str) -> Result<Self, FromHexError> {
		let bytes = hex::decode(hex).unwrap();
		Ok(Self::from_slice(&bytes).unwrap())
	}
}

pub trait ValueExtension {
	fn to_value(&self) -> Value;
}

impl ValueExtension for Bytes {
	fn to_value(&self) -> Value {
		Value::String(self.to_hex())
	}
}

impl ValueExtension for String {
	fn to_value(&self) -> Value {
		Value::String(self.clone())
	}
}

impl ValueExtension for &str {
	fn to_value(&self) -> Value {
		Value::String(self.to_string())
	}
}

impl ValueExtension for H160 {
	fn to_value(&self) -> Value {
		Value::String(self.to_string())
	}
}

impl ValueExtension for PublicKey {
	fn to_value(&self) -> Value {
		Value::String(self.to_encoded_point(false).as_bytes().to_hex())
	}
}

impl ValueExtension for H256 {
	fn to_value(&self) -> Value {
		Value::String(self.to_hex())
	}
}

impl ValueExtension for u32 {
	fn to_value(&self) -> Value {
		Value::Number(serde_json::Number::from(*self))
	}
}

impl ValueExtension for u64 {
	fn to_value(&self) -> Value {
		Value::Number(serde_json::Number::from(*self))
	}
}

impl ValueExtension for i32 {
	fn to_value(&self) -> Value {
		Value::Number(serde_json::Number::from(*self))
	}
}

impl ValueExtension for i64 {
	fn to_value(&self) -> Value {
		Value::Number(serde_json::Number::from(*self))
	}
}

impl ValueExtension for bool {
	fn to_value(&self) -> Value {
		Value::Bool(*self)
	}
}

impl ValueExtension for TransactionAttribute {
	fn to_value(&self) -> Value {
		Value::String(self.to_json())
	}
}

impl ValueExtension for TransactionSendToken {
	fn to_value(&self) -> Value {
		Value::String(self.to_json())
	}
}

impl ValueExtension for Vec<TransactionSendToken> {
	fn to_value(&self) -> Value {
		self.iter().map(|x| x.to_value()).collect()
	}
}

pub trait ExternBase64 {
	fn to_base64(&self) -> String;
}

impl ExternBase64 for String {
	fn to_base64(&self) -> String {
		general_purpose::STANDARD_NO_PAD.encode(self.as_bytes())
	}
}

// ScryptParams
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "ScryptParams")]
pub struct ScryptParamsDef {
	log_n: u8,
	r: u32,
	p: u32,
}

impl Default for ScryptParamsDef {
	fn default() -> Self {
		Self { log_n: 14, r: 8, p: 8 }
	}
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct H256Def {
	#[serde(serialize_with = "serialize_h256")]
	#[serde(deserialize_with = "deserialize_h256")]
	hash: H256,
}

// #[serde(remote = "H160")]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct H160Def {
	#[serde(serialize_with = "serialize_address")]
	#[serde(deserialize_with = "deserialize_address")]
	hash: H160,
}
