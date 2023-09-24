use crate::{
	crypto::hash::HashableForVec,
	neo_error::NeoError,
	types::{private_key::PrivateKeyExtension, Bytes},
	NeoRust,
};
use sha2::{Digest, Sha256};
use std::hash::Hash;

pub trait Wif {
	fn to_wif(&self) -> String;
	// fn from_wif(&self, s: &str) -> Option<Vec<u8>>;
}

impl Wif for &[u8] {
	fn to_wif(&self) -> String {
		if self.len() != 32 {
			return String::new()
		}

		let mut extended = vec![0x80];
		extended.extend_from_slice(self);
		extended.push(0x01);

		let hash = Sha256::digest(&Sha256::digest(&extended));
		extended.extend_from_slice(&hash[0..4]);

		bs58::encode(extended.as_slice()).into_string()
	}
}

pub trait WifExtension {
	fn to_wif(&self) -> String;

	fn from_wif(&self, s: &str) -> Result<Bytes, NeoError>;
}

pub fn str_to_wif(s: &str) -> Result<Bytes, NeoError> {
	let data = bs58::decode(s).into_vec().unwrap();

	if data.len() != 38 || data[0] != 0x80 || data[33] != 0x01 {
		return Err(NeoError::InvalidFormat)
	}

	let checksum = &data[..34].hash256().hash256()[..4];
	if checksum != &data[34..] {
		return Err(NeoError::InvalidPublicKey)
	}

	Ok(data[1..33].to_vec())
}

#[cfg(test)]
mod tests {
	use crate::types::{private_key::PrivateKeyExtension, PrivateKey};

	#[test]
	fn test_valid_wif_to_private_key() {
		let wif = "L25kgAQJXNHnhc7Sx9bomxxwVSMsZdkaNQ3m2VfHrnLzKWMLP13A";
		let expected_key = "9117f4bf9be717c9a90994326897f4243503accd06712162267e77f18b49c3a3";

		let key = PrivateKey::from_wif(wif).unwrap();
		assert_eq!(expected_key, key.to_hex_str());
	}

	#[test]
	fn test_wrongly_sized_wifs() {
		let too_large = "L25kgAQJXNHnhc7Sx9bomxxwVSMsZdkaNQ3m2VfHrnLzKWMLP13Ahc7S";
		let too_small = "L25kgAQJXNHnhc7Sx9bomxxwVSMsZdkaNQ3m2VfHrnLzKWML";

		assert!(PrivateKey::from_wif(too_large).is_err());
		assert!(PrivateKey::from_wif(too_small).is_err());
	}

	#[test]
	fn test_wrong_first_byte_wif() {
		let wif = "M25kgAQJXNHnhc7Sx9bomxxwVSMsZdkaNQ3m2VfHrnLzKWMLP13A";

		assert!(PrivateKey::from_wif(wif).is_err());
	}

	#[test]
	fn test_wrong_byte33_wif() {
		let wif = "L25kgAQJXNHnhc7Sx9bomxxwVSMsZdkaNQ3m2VfHrnLzKWMLA13A";

		assert!(PrivateKey::from_wif(wif).is_err());
	}

	#[test]
	fn test_valid_private_key_to_wif() {
		let key = "9117f4bf9be717c9a90994326897f4243503accd06712162267e77f18b49c3a3";
		let expected_wif = "L25kgAQJXNHnhc7Sx9bomxxwVSMsZdkaNQ3m2VfHrnLzKWMLP13A";

		let wif = PrivateKey::from_hex(key).unwrap().to_wif();
		assert_eq!(expected_wif, wif);
	}

	#[test]
	fn test_wrongly_sized_private_key() {
		let key = "9117f4bf9be717c9a90994326897f4243503accd06712162267e77f18b49c3";

		assert!(PrivateKey::from_hex(key).is_err());
	}
}
