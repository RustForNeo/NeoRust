use crate::{error::CryptoError, hash::HashableForVec, keys::Secp256r1PrivateKey};
use sha2::{Digest, Sha256};

/// Converts a given secret key to a Wallet Import Format (WIF) string.
///
/// # Arguments
///
/// * `secretkey` - A reference to a `Secp256r1PrivateKey` object.
///
/// # Returns
///
/// A `String` object containing the WIF representation of the given secret key.
fn prikey_to_wif(secretkey: &Secp256r1PrivateKey) -> String {
	let bytes = secretkey.to_raw_bytes();
	if bytes.len() != 32 {
		return String::new()
	}
	let mut extended = vec![0x80];
	extended.extend_from_slice(&bytes);
	extended.push(0x01);

	let hash = Sha256::digest(&Sha256::digest(&extended));
	extended.extend_from_slice(&hash[0..4]);

	bs58::encode(extended.as_slice()).into_string()
}

/// Converts a given WIF string to a private key byte vector.
///
/// # Arguments
///
/// * `s` - A reference to a `str` object containing the WIF string.
///
/// # Returns
///
/// A `Result` object containing either the private key byte vector or a `CryptoError` object.
pub fn wif_to_prikey(s: &str) -> Result<Vec<u8>, CryptoError> {
	let data = bs58::decode(s).into_vec().unwrap();

	if data.len() != 38 || data[0] != 0x80 || data[33] != 0x01 {
		return Err(CryptoError::InvalidFormat("".to_string()))
	}

	let checksum = &data[..34].hash256().hash256()[..4];
	if checksum != &data[34..] {
		return Err(CryptoError::InvalidPublicKey)
	}

	Ok(data[1..33].to_vec())
}

#[cfg(test)]
mod tests {
	use crate::wif::wif_to_prikey;

	#[test]
	fn test_valid_wif_to_private_key() {
		let wif = "L25kgAQJXNHnhc7Sx9bomxxwVSMsZdkaNQ3m2VfHrnLzKWMLP13A";
		let expected_key = "9117f4bf9be717c9a90994326897f4243503accd06712162267e77f18b49c3a3";

		let key = wif_to_prikey(wif).unwrap();
		assert_eq!(expected_key, hex::encode(key));
	}

	#[test]
	fn test_wrongly_sized_wifs() {
		let too_large = "L25kgAQJXNHnhc7Sx9bomxxwVSMsZdkaNQ3m2VfHrnLzKWMLP13Ahc7S";
		let too_small = "L25kgAQJXNHnhc7Sx9bomxxwVSMsZdkaNQ3m2VfHrnLzKWML";

		assert!(wif_to_prikey(too_large).is_err());
		assert!(wif_to_prikey(too_small).is_err());
	}

	#[test]
	fn test_wrong_first_byte_wif() {
		let wif = "M25kgAQJXNHnhc7Sx9bomxxwVSMsZdkaNQ3m2VfHrnLzKWMLP13A";

		assert!(wif_to_prikey(wif).is_err());
	}

	#[test]
	fn test_wrong_byte33_wif() {
		let wif = "L25kgAQJXNHnhc7Sx9bomxxwVSMsZdkaNQ3m2VfHrnLzKWMLA13A";

		assert!(wif_to_prikey(wif).is_err());
	}
}
