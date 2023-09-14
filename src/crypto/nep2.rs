use bitcoin::base58;
use crypto::hmac::Hmac;
use crypto::pbkdf2::pbkdf2;
use sha2::{Digest, Sha256};
use crate::crypto::scrypt_parameter::ScryptParams;

pub struct NEP2;

impl NEP2 {

    const DKLEN: usize = 64;

    fn generate_derived_key(password: &[u8], salt: &[u8], params: &ScryptParams) -> Vec<u8> {
        pbkdf2::<Hmac<Sha256>>(password, salt, params.iterations, Self::DKLEN)
    }

    fn encrypt(password: &str, private_key: &[u8]) -> String {

        let address = hash_pubkey(private_key);
        let salt = &address[..4];

        let params = ScryptParams::default();
        let derived = pbkdf2(password.as_bytes(), salt, params);

        let derived_half1 = &derived[..32];
        let derived_half2 = &derived[32..];

        let mut encrypted_half1 = private_key[..16].to_vec();
        xor(&mut encrypted_half1, derived_half1);

        let mut encrypted_half2 = private_key[16..].to_vec();
        xor(&mut encrypted_half2, derived_half2);

        let nep2 = [0x01, 0x42, 0xe0]
            .iter()
            .chain(salt)
            .chain(encrypted_half1)
            .chain(encrypted_half2)
            .collect::<Vec<u8>>();

        base58::encode(nep2.as_slice()).into_string()
    }

    fn decrypt(password: &str, nep2: &str) -> Vec<u8> {

        let data = base58::decode(nep2)
            .into_vec()
            .expect("Invalid nep2 data");

        let salt = &data[3..7];
        let params = ScryptParams::default();

        let derived = pbkdf2(password.as_bytes(), salt, params);
        let derived_half1 = &derived[..32];
        let derived_half2 = &derived[32..];

        let encrypted_half1 = &data[7..23];
        let mut decrypted_half1 = encrypted_half1.to_vec();
        xor(&mut decrypted_half1, derived_half1);

        let encrypted_half2 = &data[23..39];
        let mut decrypted_half2 = encrypted_half2.to_vec();
        xor(&mut decrypted_half2, derived_half2);

        [decrypted_half1, decrypted_half2].concat()
    }

}

fn private_key_to_address(private_key: &[u8]) -> Vec<u8> {
    let public_key = secp256k1::PublicKey::from_secret_key(private_key);
    let hash = Sha256::digest(public_key.serialize());
    hash[0..4].to_vec()
}

// Other helper functions
fn xor(output: &mut [u8], input: &[u8]) {
    for (a, b) in output.iter_mut().zip(input) {
        *a ^= b;
    }
}

fn encode_base58check(data: &[u8]) -> String {
    let checksum = sha2::Sha256::digest(&sha2::Sha256::digest(data));
    let mut result = data.to_vec();
    result.extend_from_slice(&checksum[..4]);
    base58::encode(result.as_slice()).into_string()
}

fn decode_base58check(data: &str) -> Option<Vec<u8>> {
    base58::decode(data).into_vec().ok().and_then(|bytes| {
        if bytes.len() <= 4 {
            None
        } else {
            let checksum = &bytes[bytes.len() - 4..];
            bytes.truncate(bytes.len() - 4);
            let expected = &sha2::Sha256::digest(&sha2::Sha256::digest(&bytes))[..4];
            if expected == checksum {
                Some(bytes)
            } else {
                None
            }
        }
    })
}


fn encrypt_aes128_ecb_pkcs7(data: &[u8], key: &[u8]) -> Vec<u8> {
    let cipher = Aes128::new(&key);
    let mut padding = Pkcs7::new_pad(data.len()).unwrap();
    padding.pad(data);

    let mut block_mode = Ecb::<Aes128>::new(cipher, &padding);
    let mut result = vec![0; data.len() + padding.added_len()];
    block_mode
        .encrypt(&mut result, data)
        .expect("Encryption failed");

    result
}

fn decrypt_aes128_ecb_pkcs7(data: &[u8], key: &[u8]) -> Vec<u8> {
    // Implementation omitted
}