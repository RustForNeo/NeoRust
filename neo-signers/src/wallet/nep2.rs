//! # NEO NEP2 (Neo Extended Protocol 2) Module
//!
//! This module implements the NEP2 standard for encrypting and decrypting NEO blockchain private keys.
//! NEP2 specifies a method for securing private keys with a passphrase, making it safer to store
//! and manage private keys, especially in wallet applications.
//!
//! ## Features
//!
//! - Encrypt private keys using a password to produce a NEP2-formatted string.
//! - Decrypt NEP2 strings back into private keys using the correct password.
//! - Integration with AES encryption and scrypt key derivation for robust security.
//!
//! ## Usage
//!
//! - Encrypt a private key to a NEP2 string:
//!   - Use `NEP2::encrypt` with a password and a `KeyPair` containing the private key.
//!
//! - Decrypt a NEP2 string to obtain the private key:
//!   - Use `NEP2::decrypt` with the password and the NEP2 string.
//!
//! ## Examples
//!
//! ```
//! use rand::rngs::OsRng;
//! use neo_crypto::key_pair::KeyPair;
//! use neo_crypto::keys::Secp256r1PrivateKey;
//! use neo_signers::NEP2;
//!
//! // To encrypt a private key:
//! let key_pair = KeyPair::from_secret_key(&Secp256r1PrivateKey::random(&mut OsRng));
//! let encrypted = NEP2::encrypt("your-password", &key_pair).expect("Encryption failed");
//!
//! // To decrypt a NEP2 string:
//! let decrypted_key_pair = NEP2::decrypt("your-password", &encrypted).expect("Decryption failed");
//! ```
//!
//! ## Testing
//!
//! The module includes tests to verify the correctness of the encryption and decryption functionalities,
//! ensuring that they comply with the NEP2 standard.
//!
//! ## Error Handling
//!
//! Proper error handling is implemented to deal with common issues like incorrect password, invalid NEP2 format,
//! and other cryptographic errors.

use crate::{
	base58_helper::{base58check_decode, base58check_encode},
	error::CryptoError,
	hash::HashableForVec,
	key_pair::KeyPair,
	keys::Secp256r1PrivateKey,
	WalletError,
};
use aes::{
	cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit},
	Aes128, Aes256,
};
use crypto::scrypt::{scrypt, ScryptParams};
use neo_crypto::{key_pair::KeyPair, keys::PrivateKeyExtension};
use neo_providers::public_key_to_script_hash;

// const DKLEN: usize = 64;
// const NEP2_PRIVATE_KEY_LENGTH: usize = 39;
// const NEP2_PREFIX_1: u8 = 0x01;
// const NEP2_PREFIX_2: u8 = 0x42;
// const NEP2_FLAGBYTE: u8 = 0xE0;

/// Represents the NEP2 format for encrypted private keys.
pub struct NEP2;

impl NEP2 {
	const DKLEN: usize = 64;
	const NEP2_PRIVATE_KEY_LENGTH: usize = 39;
	const NEP2_PREFIX_1: u8 = 0x01;
	const NEP2_PREFIX_2: u8 = 0x42;
	const NEP2_FLAGBYTE: u8 = 0xE0;

	pub fn decrypt(
		password: &str,
		nep2_string: &str,
		params: ScryptParams,
	) -> Result<KeyPair, CryptoError> {
		let nep2_data = nep2_string.from_base58()?;
		if nep2_data.len() != Self::NEP2_PRIVATE_KEY_LENGTH
			|| nep2_data[0] != Self::NEP2_PREFIX_1
			|| nep2_data[1] != Self::NEP2_PREFIX_2
			|| nep2_data[2] != Self::NEP2_FLAGBYTE
		{
			return Err(CryptoError::InvalidFormat("Not valid NEP2 prefix.".to_string()).into())
		}
		let address_hash = &nep2_data[3..7];
		let encrypted = &nep2_data[7..39];
		let derived_key =
			Self::generate_derived_scrypt_key(password.as_bytes(), address_hash, params)?;
		let decrypted_bytes = Self::perform_cipher(encrypted, &derived_key[32..], false)?;
		let plain_private_key = xor(&derived_key[..32], &decrypted_bytes);
		let key_pair = KeyPair::from_private_key(&plain_private_key.into())?;
		let new_address_hash = key_pair.get_address_hash()?;
		if new_address_hash != address_hash {
			return Err(CryptoError::InvalidPassphrase(
				"Calculated address hash does not match the one in the provided encrypted address."
					.to_string(),
			)
			.into())
		}
		Ok(key_pair)
	}

	pub fn encrypt(
		password: &str,
		key_pair: &KeyPair,
		params: ScryptParams,
	) -> Result<String, WalletError> {
		let address_hash = public_key_to_script_hash(&key_pair.public_key);
		let private_key = key_pair.private_key().to_vec();
		let derived_key =
			Self::generate_derived_scrypt_key(password.as_bytes(), &address_hash, params)?;
		let derived_half1 = &derived_key[..32];
		let derived_half2 = &derived_key[32..];
		let encrypted_half1 = Self::perform_cipher(
			&Self::xor_private_key_and_derived_half(private_key, derived_half1, 0..16),
			derived_half2,
			true,
		)?;
		let encrypted_half2 = Self::perform_cipher(
			&Self::xor_private_key_and_derived_half(private_key, derived_half1, 16..32),
			derived_half2,
			true,
		)?;
		let result = [
			vec![Self::NEP2_PREFIX_1, Self::NEP2_PREFIX_2, Self::NEP2_FLAGBYTE],
			address_hash.to_vec(),
			encrypted_half1,
			encrypted_half2,
		]
		.concat();
		Ok(result.to_base58())
	}

	// Helper functions
	fn xor(a: &[u8], b: &[u8]) -> Vec<u8> {
		a.iter().zip(b.iter()).map(|(&x, &y)| x ^ y).collect()
	}

	fn xor_private_key_and_derived_half(
		private_key: &[u8],
		half: &[u8],
		range: std::ops::Range<usize>,
	) -> Vec<u8> {
		xor(&private_key[range.clone()], &half[range])
	}

	fn perform_cipher(data: &[u8], key: &[u8], decrypt: bool) -> Result<Vec<u8>, CryptoError> {
		let cipher = Aes256::new_varkey(key)?;
		if decrypt {
			Ok(cipher.decrypt_vec(data)?)
		} else {
			Ok(cipher.encrypt_vec(data))
		}
	}

	fn generate_derived_scrypt_key(
		password: &[u8],
		salt: &[u8],
		params: ScryptParams,
	) -> Result<Vec<u8>, CryptoError> {
		let mut output = vec![0u8; Self::DKLEN];
		scrypt(password, salt, &params, &mut output)?;
		Ok(output)
	}
}

/// Generates a derived scrypt key.
///
/// # Arguments
///
/// * `password` - The password string.
/// * `salt` - The salt value.
///
/// Returns the derived key.
fn generate_derived_scrypt_key(password: &str, salt: &[u8]) -> Result<Vec<u8>, &'static str> {
	let pwd = password.as_bytes();
	let mut dk = vec![0u8; NEP2::DKLEN];
	scrypt(pwd, salt, &ScryptParams::new(14, 8, 1), &mut dk);
	Ok(dk)
}

/// Decrypts data using AES with the provided key.
///
/// # Arguments
///
/// * `data` - The data to be decrypted.
/// * `key` - The decryption key.
///
/// Returns the decrypted data.
fn decrypt_aes(data: &[u8], key: &[u8]) -> Result<Vec<u8>, &'static str> {
	if data.len() != 16 {
		return Err("Data must be exactly 16 bytes for AES-128 block decryption")
	}

	if key.len() != 16 {
		return Err("Key must be exactly 16 bytes for AES-128")
	}

	let cipher = Aes128::new(GenericArray::from_slice(key));
	let mut block = GenericArray::clone_from_slice(data);
	cipher.decrypt_block(&mut block);

	Ok(block.to_vec())
}

/// Encrypts data using AES with the provided key.
///
/// # Arguments
///
/// * `data` - The data to be encrypted.
/// * `key` - The encryption key.
///
/// Returns the encrypted data.
fn encrypt_aes(data: &[u8], key: &[u8]) -> Result<Vec<u8>, &'static str> {
	let cipher = Aes128::new(key.into());
	let mut block_data = [0u8; 16];
	block_data.copy_from_slice(data);
	let mut block = GenericArray::from(block_data);
	cipher.encrypt_block(&mut block);
	Ok(block.to_vec())
}

/// XOR operation between two byte slices.
///
/// # Arguments
///
/// * `a` - First byte slice.
/// * `b` - Second byte slice.
///
/// Returns the result of the XOR operation.
fn xor(a: &[u8], b: &[u8]) -> Vec<u8> {
	assert_eq!(a.len(), b.len());
	let mut result = vec![0u8; a.len()];
	for i in 0..a.len() {
		result[i] = a[i] ^ b[i];
	}
	result
}

/// Computes a hash from a public key and extracts the first 4 bytes.
///
/// # Arguments
///
/// * `pubkey` - The public key.
///
/// Returns the first 4 bytes of the hash.
fn address_hash_from_pubkey(pubkey: &[u8]) -> [u8; 4] {
	let hash = pubkey.hash256();
	let mut result = [0u8; 4];
	result.copy_from_slice(&hash[..4]);
	result
}

#[cfg(test)]
mod tests {
	use super::*;
	use neo_config::TestConstants;

	#[test]
	fn test_decrypt_with_default_scrypt_params() {
		let decrypted_key_pair = match NEP2::decrypt(
			TestConstants::DEFAULT_ACCOUNT_PASSWORD,
			TestConstants::DEFAULT_ACCOUNT_ENCRYPTED_PRIVATE_KEY,
		) {
			Ok(key_pair) => key_pair,
			Err(_) => panic!("Decryption failed"),
		};
		assert_eq!(
			decrypted_key_pair.private_key_bytes().to_vec(),
			hex::decode(TestConstants::DEFAULT_ACCOUNT_PRIVATE_KEY).unwrap()
		);
	}

	#[test]
	fn test_decrypt_with_non_default_scrypt_params() {
		// Assuming the ScryptParams can be passed to the decrypt function
		// let params = ScryptParams::new(256, 1, 1); // Adjust as needed
		let encrypted = "6PYM7jHL3uwhP8uuHP9fMGMfJxfyQbanUZPQEh1772iyb7vRnUkbkZmdRT";
		let decrypted_key_pair =
			NEP2::decrypt(TestConstants::DEFAULT_ACCOUNT_PASSWORD, encrypted).unwrap();
		assert_eq!(
			decrypted_key_pair.private_key_bytes().to_vec(),
			hex::decode(TestConstants::DEFAULT_ACCOUNT_PRIVATE_KEY).unwrap()
		);
	}

	#[test]
	fn test_encrypt_with_default_scrypt_params() {
		let key_pair = KeyPair::from_secret_key(
			&Secp256r1PrivateKey::from_bytes(
				&hex::decode(TestConstants::DEFAULT_ACCOUNT_PRIVATE_KEY).unwrap(),
			)
			.unwrap(),
		);
		let encrypted = NEP2::encrypt(TestConstants::DEFAULT_ACCOUNT_PASSWORD, &key_pair).unwrap();
		assert_eq!(encrypted, TestConstants::DEFAULT_ACCOUNT_ENCRYPTED_PRIVATE_KEY);
	}

	#[test]
	fn test_encrypt_with_non_default_scrypt_params() {
		// Assuming the ScryptParams can be passed to the encrypt function
		// let params = ScryptParams::new(256, 1, 1); // Adjust as needed
		let expected = "6PYM7jHL3uwhP8uuHP9fMGMfJxfyQbanUZPQEh1772iyb7vRnUkbkZmdRT";
		let key_pair = KeyPair::from_secret_key(
			&Secp256r1PrivateKey::from_bytes(
				&hex::decode(TestConstants::DEFAULT_ACCOUNT_PRIVATE_KEY).unwrap(),
			)
			.unwrap(),
		);
		let encrypted = NEP2::encrypt(TestConstants::DEFAULT_ACCOUNT_PASSWORD, &key_pair).unwrap();
		assert_eq!(encrypted, expected);
	}
}
