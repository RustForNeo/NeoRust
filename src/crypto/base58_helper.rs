use sha2::{Digest, Sha256};

pub fn base58check_encode(bytes: &[u8]) -> String {
	let checksum = &calculate_checksum(bytes)[..4];
	let bytes_with_checksum = [bytes, checksum].concat();
	bs58::encode(bytes_with_checksum).into_string()
}

pub fn base58check_decode(input: &str) -> Option<Vec<u8>> {
	let bytes_with_checksum = bs58::decode(input).into_vec().ok().unwrap();

	let bytes = &bytes_with_checksum[..bytes_with_checksum.len() - 4];
	let checksum = &bytes_with_checksum[bytes_with_checksum.len() - 4..];

	let expected_checksum = &calculate_checksum(bytes)[..4];
	if checksum != expected_checksum {
		return None
	}

	Some(bytes.to_vec())
}

fn calculate_checksum(input: &[u8]) -> [u8; 4] {
	let mut hasher = Sha256::new();
	hasher.update(input);
	let hash = hasher.finalize();
	let hash256 = Sha256::digest(&hash);
	hash256[..4].try_into().unwrap()
}
