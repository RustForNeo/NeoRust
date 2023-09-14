use crypto::digest::Digest;
use crypto::scrypt::{scrypt, ScryptParams};
use crypto::sha2::Sha256;
use futures::TryFutureExt;
use p256::ecdsa::SigningKey;

const DKLEN: usize = 64;
const NEP2_PRIVATE_KEY_LENGTH: usize = 39;
const NEP2_PREFIX_1: u8 = 0x01;
const NEP2_PREFIX_2: u8 = 0x42;
const NEP2_FLAGBYTE: u8 = 0xE0;

pub struct NEP2;

impl NEP2 {

    pub fn decrypt(password: &str, nep2_string: &str) -> Result<SigningKey, &'static str> {

        let nep2_data = nep2_string.from_base58check()?;

        if nep2_data.len() != NEP2_PRIVATE_KEY_LENGTH {
            return Err("Invalid NEP2 length");
        }

        if nep2_data[0] != NEP2_PREFIX_1 || nep2_data[1] != NEP2_PREFIX_2 || nep2_data[2] != NEP2_FLAGBYTE {
            return Err("Invalid NEP2 prefix");
        }

        let address_hash = &nep2_data[3..7];
        let encrypted = &nep2_data[7..39];

        let derived_key = derive_scrypt_key(password, address_hash)?;
        let derived_half1 = &derived_key[..32];
        let derived_half2 = &derived_key[32..];

        let decrypted_half1 = aes_decrypt(encrypted[..16], derived_half2)?;
        let decrypted_half2 = aes_decrypt(&encrypted[16..], derived_half2)?;

        let private_key = xor_keys(&decrypted_half1, derived_half1)
            .chain(xor_keys(&decrypted_half2, derived_half2))
            .collect::<Vec<_>>();

        let key_pair = SigningKey::new(&Secp256k1::new(), &private_key);

        let new_address_hash = address_hash_from_pubkey(&key_pair.verifying_key)?;

        if new_address_hash != address_hash {
            return Err("Invalid passphrase");
        }

        Ok(key_pair)
    }

    pub fn encrypt(password: &str, key_pair: &SigningKey) -> Result<String, &'static str> {

        let address_hash = address_hash_from_pubkey(&key_pair.verifying_key)?;

        let private_key = key_pair.to_bytes();

        let derived_key = derive_scrypt_key(password, &address_hash)?;
        let derived_half1 = &derived_key[..32];
        let derived_half2 = &derived_key[32..];

        let encrypted_half1 = aes_encrypt(xor_keys(&private_key[..16], derived_half1), derived_half2)?;
        let encrypted_half2 = aes_encrypt(xor_keys(&private_key[16..], derived_half1), derived_half2)?;

        let mut nep2_data = vec![NEP2_PREFIX_1, NEP2_PREFIX_2, NEP2_FLAGBYTE];
        nep2_data.extend_from_slice(address_hash);
        nep2_data.extend_from_slice(&encrypted_half1);
        nep2_data.extend_from_slice(&encrypted_half2);

        let nep2_string = nep2_data.to_base58check();

        Ok(nep2_string)
    }

    fn derive_scrypt_key(password: &str, salt: &[u8]) -> Result<Vec<u8>, &'static str> {
        let params = ScryptParams::new(14, 8, 8)?;
        let mut hash = [0u8; DKLEN];
        let _ = scrypt(password.as_bytes(), salt, &params, &mut hash).map_err(|_| "Scrypt error");
        Ok(hash.to_vec())
    }

    fn aes_encrypt(data: Vec<u8>, key: &[u8]) -> Result<Vec<u8>, &'static str> {
        let cipher = Aes128::new(key);
        cipher.encrypt_vec(data).map_err(|_| "AES encrypt error")
    }

    fn aes_decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, &'static str> {
        let cipher = Aes128::new(key);
        cipher.decrypt_vec(data).map_err(|_| "AES decrypt error")
    }

    fn xor_keys(a: &[u8], b: &[u8]) -> impl Iterator<Item = u8> {
        a.iter().zip(b).map(|(x, y)| x ^ y)
    }

    fn address_hash_from_pubkey(pubkey: &[u8]) -> Result<Vec<u8>, &'static str> {
        let mut hasher = Sha256::new();
        hasher.input(pubkey);
        Ok(hasher.result(&mut [])[..4].to_vec())
    }

}