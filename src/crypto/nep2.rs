use crate::{
	crypto::{
		base58_helper::{base58check_decode, base58check_encode},
		hash::HashableForVec,
		key_pair::KeyPair,
	},
	types::{private_key::PrivateKeyExtension, public_key::PublicKeyExtension, PrivateKey},
};
use aes::{
	cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit},
	Aes128,
};
use crypto::scrypt::{scrypt, ScryptParams};

const DKLEN: usize = 64;
const NEP2_PRIVATE_KEY_LENGTH: usize = 39;
const NEP2_PREFIX_1: u8 = 0x01;
const NEP2_PREFIX_2: u8 = 0x42;
const NEP2_FLAGBYTE: u8 = 0xE0;

pub struct NEP2;

impl NEP2 {
	pub fn decrypt(password: &str, nep2_string: &str) -> Result<KeyPair, &'static str> {
		let nep2_data = base58check_decode(nep2_string).unwrap();

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

		let derived_key = generate_derived_scrypt_key(password, address_hash).unwrap();

		let decrypted_bytes = decrypt_aes(encrypted, &derived_key[..32]).unwrap();

		let plain_private_key = xor(&decrypted_bytes, &derived_key[..32]);

		let private_key = PrivateKey::from_bytes(&plain_private_key).unwrap();

		let key_pair = KeyPair::from_private_key(private_key);
		let new_address_hash = address_hash_from_pubkey(&key_pair.public_key().to_vec());

		if new_address_hash != address_hash {
			return Err("Invalid passphrase")
		}

		Ok(key_pair)
	}

	pub fn encrypt(password: &str, key_pair: &KeyPair) -> Result<String, &'static str> {
		let address_hash = address_hash_from_pubkey(&key_pair.public_key().to_vec());

		let private_key = key_pair.private_key().to_vec();

		let derived_key = generate_derived_scrypt_key(password, &address_hash)?;

		let derived_half1 = &derived_key[..32];
		let derived_half2 = &derived_key[32..];

		let encrypted_half1 = encrypt_aes(&xor(&private_key[..16], derived_half1), derived_half2)?;
		let encrypted_half2 =
			encrypt_aes(&xor(&private_key[16..32], derived_half1), derived_half2)?;

		let mut result = vec![NEP2_PREFIX_1, NEP2_PREFIX_2, NEP2_FLAGBYTE];
		result.extend_from_slice(&address_hash);
		result.extend_from_slice(&encrypted_half1);
		result.extend_from_slice(&encrypted_half2);

		Ok(base58check_encode(&result))
	}
}

fn generate_derived_scrypt_key(password: &str, salt: &[u8]) -> Result<Vec<u8>, &'static str> {
	let pwd = password.as_bytes();
	let mut dk = vec![0u8; DKLEN];

	scrypt(pwd, salt, &ScryptParams::new(14, 8, 1), &mut dk);

	Ok(dk)
}

fn decrypt_aes(data: &[u8], key: &[u8]) -> Result<Vec<u8>, &'static str> {
	let cipher = Aes128::new(key.into());
	let mut block_data = [0u8; 16]; //data.iter().try_into().expect("slice with incorrect length");
	block_data.copy_from_slice(data);
	let mut block = GenericArray::from(block_data);
	cipher.decrypt_block(&mut block);
	Ok(block.to_vec())
}

fn encrypt_aes(data: &[u8], key: &[u8]) -> Result<Vec<u8>, &'static str> {
	let cipher = Aes128::new(key.into());
	let mut block_data = [0u8; 16]; //data.iter().try_into().expect("slice with incorrect length");
	block_data.copy_from_slice(data);
	let mut block = GenericArray::from(block_data);
	cipher.encrypt_block(&mut block);
	Ok(block.to_vec())
}

fn xor(a: &[u8], b: &[u8]) -> Vec<u8> {
	assert_eq!(a.len(), b.len());
	let mut result = vec![0u8; a.len()];
	for i in 0..a.len() {
		result[i] = a[i] ^ b[i];
	}
	result
}

fn address_hash_from_pubkey(pubkey: &[u8]) -> [u8; 4] {
	let hash = pubkey.hash256();
	let mut result = [0u8; 4];
	result.copy_from_slice(&hash[..4]);
	result
}
