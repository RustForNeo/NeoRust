use crate::{error::SignerError, script::script_builder::ScriptBuilder};
use neo_config::DEFAULT_ADDRESS_VERSION;
use neo_crypto::{
	hash::HashableForVec,
	keys::{PrivateKeyExtension, PublicKeyExtension},
};
use neo_types::script_hash::{ScriptHash, ScriptHashExtension};
use p256::{PublicKey, SecretKey};
use primitive_types::H160;
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
pub fn public_key_to_script_hash(public_key: &PublicKey) -> ScriptHash {
	let mut script = ScriptBuilder::build_verification_script(public_key).unwrap();
	script_hash_from_script(&script)
}

/// Convert a public key to an address.
pub fn public_key_to_address(public_key: &PublicKey) -> String {
	let script_hash = public_key_to_script_hash(public_key);
	script_hash_to_address(&script_hash)
}

/// Convert a private key to a public key.
pub fn private_key_to_public_key(private_key: &SecretKey) -> PublicKey {
	PublicKey::from_secret_key(private_key)
}

/// Convert a private key to a script hash.
pub fn private_key_to_script_hash(private_key: &SecretKey) -> ScriptHash {
	let pubkey = private_key_to_public_key(private_key);
	public_key_to_script_hash(&pubkey)
}

/// Convert a private key to an address.
pub fn private_key_to_address(private_key: &SecretKey) -> String {
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

/// Convert a private key in WIF format to a SecretKey.
pub fn private_key_from_wif(wif: &str) -> Result<SecretKey, SignerError> {
	let secret_key = SecretKey::from_wif(wif)?;
	Ok(secret_key)
}

/// Convert a private key to WIF format.
pub fn private_key_to_wif(private_key: &SecretKey) -> String {
	private_key.to_wif()
}

/// Convert a private key to hex format.
pub fn private_key_to_hex(private_key: &SecretKey) -> String {
	private_key.to_vec().to_hex()
}

/// Convert a private key in hex format to a SecretKey.
pub fn private_key_from_hex(hex: &str) -> Result<SecretKey, SignerError> {
	let bytes = hex::decode(hex)?;
	let secret_key = SecretKey::from_slice(&bytes)?;
	Ok(secret_key)
}

/// Convert a public key to hex format.
pub fn public_key_to_hex(public_key: &PublicKey) -> String {
	public_key.to_vec().to_hex()
}

/// Convert a public key in hex format to a PublicKey.
pub fn public_key_from_hex(hex: &str) -> Result<PublicKey, SignerError> {
	let bytes = hex::decode(hex)?;
	let public_key = PublicKey::from_slice(&bytes)?;
	Ok(public_key)
}

/// Convert a script hash to hex format.
pub fn script_hash_to_hex(script_hash: &ScriptHash) -> String {
	script_hash.to_string()
}

/// Convert a script hash in hex format to a ScriptHash.
pub fn script_hash_from_hex(hex: &str) -> Result<ScriptHash, SignerError> {
	let script_hash = H160::from_str(hex)?;
	Ok(script_hash)
}

/// Convert an address to hex format.
pub fn address_to_hex(address: &str) -> Result<String, SignerError> {
	let script_hash = H160::from_address(address)?;
	Ok(script_hash.to_string())
}

/// Convert a hex format script hash to an address.
pub fn hex_to_address(hex: &str) -> Result<String, SignerError> {
	let script_hash = H160::from_str(hex)?;
	Ok(script_hash.to_address())
}
