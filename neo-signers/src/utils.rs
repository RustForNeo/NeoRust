use crate::error::SignerError;
use neo_config::DEFAULT_ADDRESS_VERSION;
use neo_crypto::{
	hash::HashableForVec,
	keys::{PrivateKeyExtension, PublicKeyExtension, Secp256r1PrivateKey, Secp256r1PublicKey},
};
use neo_providers::core::script::script_builder::ScriptBuilder;
use neo_types::script_hash::{ScriptHash, ScriptHashExtension};
use primitive_types::H160;
use rustc_serialize::hex::ToHex;
use std::str::FromStr;

/// Convert a script to a script hash.
pub fn script_hash_from_script(script: &[u8]) -> ScriptHash {
	let mut hash = script.sha256_ripemd160();
	hash.reverse();
	let mut arr = [0u8; 20];
	arr.copy_from_slice(&hash);
	H160::from(arr)
}

/// Convert a public key to a script hash.
pub fn public_key_to_script_hash(public_key: &Secp256r1PublicKey) -> ScriptHash {
	let mut script = ScriptBuilder::build_verification_script(public_key);
	script_hash_from_script(&script)
}

/// Convert a public key to an address.
pub fn public_key_to_address(public_key: &Secp256r1PublicKey) -> String {
	let script_hash = public_key_to_script_hash(public_key);
	script_hash_to_address(&script_hash)
}

/// Convert a private key to a public key.
pub fn private_key_to_public_key(private_key: &Secp256r1PrivateKey) -> Secp256r1PublicKey {
	private_key.to_public_key().unwrap()
}

/// Convert a private key to a script hash.
pub fn private_key_to_script_hash(private_key: &Secp256r1PrivateKey) -> ScriptHash {
	let pubkey = private_key_to_public_key(private_key);
	public_key_to_script_hash(&pubkey)
}

/// Convert a private key to an address.
pub fn private_key_to_address(private_key: &Secp256r1PrivateKey) -> String {
	let script_hash = private_key_to_script_hash(private_key);
	script_hash_to_address(&script_hash)
}

/// Convert a script hash to an address.
pub fn script_hash_to_address(script_hash: &ScriptHash) -> String {
	let mut data = vec![DEFAULT_ADDRESS_VERSION];
	data.extend_from_slice(&script_hash.0);
	let mut sha = &data.hash256().hash256();
	data.extend_from_slice(&sha[..4]);
	bs58::encode(data).into_string()
}

/// Convert an address to a script hash.
pub fn address_to_script_hash(address: &str) -> Result<ScriptHash, SignerError> {
	let bytes = match bs58::decode(address).into_vec() {
		Ok(bytes) => bytes,
		Err(_) => return Err(SignerError::InvalidAddress),
	};
	let salt = bytes[0];
	let hash = &bytes[1..21];
	let checksum = &bytes[21..25];
	let mut sha = &bytes[..21].hash256().hash256();
	let check = &sha[..4];
	if checksum != check {
		return Err(SignerError::InvalidAddress)
		// panic!("Invalid address checksum");
	}

	let mut rev = [0u8; 20];
	rev.clone_from_slice(hash);
	rev.reverse();
	Ok(H160::from(&rev))
}

/// Convert a private key in WIF format to a Secp256r1PrivateKey.
pub fn private_key_from_wif(wif: &str) -> Result<Secp256r1PrivateKey, SignerError> {
	private_key_from_wif(wif)
}

/// Convert a private key to WIF format.
pub fn private_key_to_wif(private_key: &Secp256r1PrivateKey) -> String {
	private_key_to_wif(private_key)
}

/// Convert a private key to hex format.
pub fn private_key_to_hex(private_key: &Secp256r1PrivateKey) -> String {
	private_key.to_raw_bytes().to_vec().to_hex()
}

/// Convert a private key in hex format to a Secp256r1PrivateKey.
pub fn private_key_from_hex(hex: &str) -> Result<Secp256r1PrivateKey, SignerError> {
	let bytes = hex::hex::decode(hex)?;
	let secret_key = Secp256r1PrivateKey::from_slice(&bytes)?;
	Ok(secret_key)
}

/// Convert a public key to hex format.
pub fn public_key_to_hex(public_key: &Secp256r1PublicKey) -> String {
	public_key.to_vec().to_hex()
}

/// Convert a public key in hex format to a Secp256r1PublicKey.
pub fn public_key_from_hex(hex: &str) -> Result<Secp256r1PublicKey, SignerError> {
	let bytes = hex::hex::decode(hex)?;
	let public_key = Secp256r1PublicKey::from_slice(&bytes)?;
	Ok(public_key)
}

/// Convert a script hash to hex format.
pub fn script_hash_to_hex(script_hash: &ScriptHash) -> String {
	let bytes: [u8; 20] = script_hash.to_fixed_bytes();
	hex::encode(bytes)
}

/// Convert a script hash in hex format to a ScriptHash.
pub fn script_hash_from_hex(hex: &str) -> Result<ScriptHash, SignerError> {
	H160::from_str(hex).map_err(|_| SignerError::InvalidAddress)
}

/// Convert an address to hex format.
pub fn address_to_hex(address: &str) -> Result<String, SignerError> {
	let script_hash = H160::from_address(address)?;
	Ok(hex::encode(script_hash.to_fixed_bytes()))
}

/// Convert a hex format script hash to an address.
pub fn hex_to_address(hex: &str) -> Result<String, SignerError> {
	let script_hash = H160::from_str(hex).map_err(|_| SignerError::InvalidAddress)?;
	Ok(script_hash.to_address())
}
