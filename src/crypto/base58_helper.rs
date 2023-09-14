use bitcoin::base58;
use sha2::{Sha256, Digest};

pub fn base58check_encode(input: &[u8]) -> String {
    let checksum = calculate_checksum(input);

    let mut result = input.to_vec();
    result.extend_from_slice(&checksum);

    base58::encode_check(&result)
}

pub fn base58check_decode(input: &str) -> Option<Vec<u8>> {
    base58::decode_check(input)
        .ok()
        .and_then(|bytes| {
            if bytes.len() <= 4 {
                None
            } else {
                let checksum = &bytes[(bytes.len() - 4)..];
                let data = &bytes[..(bytes.len() - 4)];
                let expected = calculate_checksum(data);
                if expected == checksum {
                    Some(data.to_vec())
                } else {
                    None
                }
            }
        })
}

fn calculate_checksum(input: &[u8]) -> [u8; 4] {
    let mut hasher = Sha256::new();
    hasher.update(input);

    let hash1 = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(&hash1[..]);

    let hash2 = hasher.finalize();

    let mut checksum = [0u8; 4];
    checksum.copy_from_slice(&hash2[..4]);
    checksum
}