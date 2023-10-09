use sha2::{Digest, Sha256};

/// Encodes a byte slice into a base58check string.
///
/// # Arguments
///
/// * `bytes` - A byte slice to be encoded.
///
/// # Example
///
/// ```
/// use neo_crypto::base58_helper::base58check_encode;
///
/// let bytes = [0x01, 0x02, 0x03];
/// let encoded = base58check_encode(&bytes);
/// ```
pub fn base58check_encode(bytes: &[u8]) -> String {
	let checksum = &calculate_checksum(bytes)[..4];
	let bytes_with_checksum = [bytes, checksum].concat();
	bs58::encode(bytes_with_checksum).into_string()
}

/// Decodes a base58check string into a byte vector.
///
/// # Arguments
///
/// * `input` - A base58check string to be decoded.
///
/// # Example
///
/// ```
/// use neo_crypto::base58_helper::base58check_decode;
///
/// let input = "Abc123";
/// let decoded = base58check_decode(input);
/// ```
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

/// Calculates the checksum of a byte slice.
///
/// # Arguments
///
/// * `input` - A byte slice to calculate the checksum for.
///
/// # Example
///
/// ```
/// use neo_crypto::base58_helper::calculate_checksum;
///
/// let bytes = [0x01, 0x02, 0x03];
/// let checksum = calculate_checksum(&bytes);
/// ```
pub fn calculate_checksum(input: &[u8]) -> [u8; 4] {
	let mut hasher = Sha256::new();
	hasher.update(input);
	let hash = hasher.finalize();
	let hash256 = Sha256::digest(&hash);
	hash256[..4].try_into().unwrap()
}
