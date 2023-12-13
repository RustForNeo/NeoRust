use neo_crypto::hash::HashableForVec;
use rand::Rng;

pub type Address = String;

pub trait AddressExtension {
	fn to_script_hash(&self) -> Result<Vec<u8>, &'static str>;

	fn random() -> Self;
}

impl AddressExtension for String {
	fn to_script_hash(&self) -> Result<Vec<u8>, &'static str> {
		// Base58-decode the address
		let decoded_data = match bs58::decode(self).into_vec() {
			Ok(data) => data,
			Err(_) => return Err("Failed to decode Base58"),
		};

		// Extract the data payload
		let data_payload = decoded_data[1..decoded_data.len() - 4].to_vec();

		let script_hash = data_payload.sha256_ripemd160(); //  ripemd160.finalize();

		Ok(script_hash)
	}

	fn random() -> Self {
		let mut rng = rand::thread_rng();
		let mut bytes = [0u8; 20];
		rng.fill(&mut bytes);
		let script_hash = bytes.sha256_ripemd160();
		let mut data = vec![0x17];
		data.extend_from_slice(&script_hash);
		let mut sha = &data.hash256().hash256();
		data.extend_from_slice(&sha[..4]);
		bs58::encode(data).into_string()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_address_to_script_hash() {
		// Test case 1: Valid N3 address
		let n3_address = "NTGYC16CN5QheM4ZwfhUp9JKq8bMjWtcAp";
		let expected_script_hash_hex = "87c06be672d5600dce4a260e7b2d497112c0ac50";
		let result = n3_address.to_string().to_script_hash().unwrap();
		assert_eq!(hex::encode(result), expected_script_hash_hex);

		// Test case 3: Invalid N3 address
		let n3_address = "Invalid_Address";
		let result = n3_address.to_string().to_script_hash();
		assert!(result.is_err());
	}
}
