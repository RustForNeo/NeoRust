use crate::{
	crypto::base58_helper::base58check_decode,
	types::{PrivateKey, PublicKey},
};
use aes::{cipher::KeyInit, Aes128};
use crypto::{
	digest::Digest,
	scrypt::{scrypt, ScryptParams},
	sha2::Sha256,
};
use futures::TryFutureExt;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use rayon::prelude::*;

const DKLEN: usize = 64;
const NEP2_PRIVATE_KEY_LENGTH: usize = 39;
const NEP2_PREFIX_1: u8 = 0x01;
const NEP2_PREFIX_2: u8 = 0x42;
const NEP2_FLAGBYTE: u8 = 0xE0;

pub struct NEP2;

impl NEP2 {
	pub fn decrypt(password: &str, nep2_string: &str) -> Result<PrivateKey, &'static str> {
		let nep2_data = base58check_decode(nep2_string).unwrap(); //nep2_string.from_base58check().unwrap();

		if nep2_data.len() != NEP2_PRIVATE_KEY_LENGTH {
			return Err("Invalid NEP2 length")
		}

		if nep2_data[0] != NEP2_PREFIX_1
			|| nep2_data[1] != NEP2_PREFIX_2
			|| nep2_data[2] != NEP2_FLAGBYTE
		{
			return Err("Invalid NEP2 prefix")
		}

		let address_hash = &nep2_data[3..7];
		let encrypted = &nep2_data[7..39];

		let derived_key = Self::derive_scrypt_key(password, address_hash).unwrap();
		let derived_half1 = &derived_key[..32];
		let derived_half2 = &derived_key[32..];

		let decrypted_half1 = Self::aes_decrypt(&encrypted[..16], derived_half2).unwrap();
		let decrypted_half2 = Self::aes_decrypt(&encrypted[16..], derived_half2).unwrap();

		let private_key = Self::xor_keys(&decrypted_half1, derived_half1)
			.chain(Self::xor_keys(&decrypted_half2, derived_half2))
			.collect::<Vec<_>>();

		let private_key = PrivateKey::from_bytes(private_key.as_slice()).unwrap();
		let public_key = PublicKey::from(private_key);

		let new_address_hash =
			Self::address_hash_from_pubkey(public_key.to_encoded_point(true).as_bytes()).unwrap();

		if new_address_hash != address_hash {
			return Err("Invalid passphrase")
		}

		Ok(private_key.clone())
	}

	pub fn encrypt(password: &str, private_key: &PrivateKey) -> Result<String, &'static str> {
		let public_key = PublicKey::from(private_key);
		let address_hash =
			Self::address_hash_from_pubkey(public_key.to_encoded_point(true).as_bytes()).unwrap();
		let derived_key = Self::derive_scrypt_key(password, &address_hash).unwrap();
		let derived_half1 = &derived_key[..32];
		let derived_half2 = &derived_key[32..];

		let encrypted_half1 =
			Self::aes_encrypt(Self::xor_keys(&private_key[..16], derived_half1), derived_half2)
				.unwrap();
		let encrypted_half2 =
			Self::aes_encrypt(Self::xor_keys(&private_key[16..], derived_half1), derived_half2)
				.unwrap();

		let mut nep2_data = vec![NEP2_PREFIX_1, NEP2_PREFIX_2, NEP2_FLAGBYTE];
		nep2_data.extend_from_slice(address_hash.as_slice());
		nep2_data.extend_from_slice(&encrypted_half1);
		nep2_data.extend_from_slice(&encrypted_half2);

		let nep2_string = nep2_data.to_base58check();

		Ok(nep2_string)
	}

	fn derive_scrypt_key(password: &str, salt: &[u8]) -> Result<Vec<u8>, &'static str> {
		let params = ScryptParams::new(14, 8, 1).unwrap();
		let mut hash = [0u8; DKLEN];
		let _ = scrypt(password.as_bytes(), salt, &params, &mut hash).map_err(|_| "Scrypt error");
		Ok(hash.to_vec())
	}

	fn aes_encrypt(data: Vec<u8>, key: &[u8]) -> Result<Vec<u8>, &'static str> {
		let cipher = Aes128::new(key.into());
		cipher.encrypt_vec(data).map_err(|_| "AES encrypt error")
	}

	fn aes_decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, &'static str> {
		let cipher = Aes128::new(key.into());
		cipher.decrypt_vec(data).map_err(|_| "AES decrypt error")
	}

	fn xor_keys(a: &[u8], b: &[u8]) -> impl Iterator<Item = u8> {
		a.par_iter().zip(b).map(|(x, y)| x ^ y)
	}

	fn address_hash_from_pubkey(pubkey: &[u8]) -> Result<Vec<u8>, &'static str> {
		let mut hasher = Sha256::new();
		hasher.input(pubkey);
		Ok(hasher.result(&mut [])[..4].to_vec())
	}
}
