use crypto::{
	digest::Digest,
	hmac::Hmac,
	mac::Mac,
	ripemd160::Ripemd160,
	sha2::{Sha256, Sha512},
};

pub trait HashableForVec {
	fn hash256(&self) -> Vec<u8>;
	fn ripemd160(&self) -> Vec<u8>;
	fn sha256_ripemd160(&self) -> Vec<u8>;
	fn hmac_sha512(&self, key: &[u8]) -> Vec<u8>;
}

impl HashableForVec for [u8] {
	fn hash256(&self) -> Vec<u8> {
		let mut hasher = Sha256::new();
		hasher.input(self);
		let mut res = vec![0u8; 32];
		hasher.result(&mut res);
		res
	}

	fn ripemd160(&self) -> Vec<u8> {
		let mut hasher = Ripemd160::new();
		hasher.input(self);
		let mut res = vec![0u8; 20];
		hasher.result(&mut res);

		res
	}

	fn sha256_ripemd160(&self) -> Vec<u8> {
		let mut sha256 = Sha256::new();
		sha256.input(self);
		let mut res = vec![0u8; 32];
		sha256.result(&mut res);
		let mut hasher = Ripemd160::new();
		hasher.input(&res);
		let mut res = vec![0u8; 20];
		hasher.result(&mut res);
		res
	}

	fn hmac_sha512(&self, key: &[u8]) -> Vec<u8> {
		let mut hmac = Hmac::new(Sha512::new(), key);

		hmac.input(self);
		let res = hmac.result();
		res.code().to_vec()
	}
}

impl HashableForVec for Vec<u8> {
	fn hash256(&self) -> Vec<u8> {
		let mut hasher = Sha256::new();
		hasher.input(self);
		let mut res = vec![0u8; 32];
		hasher.result(&mut res);
		res
	}

	fn ripemd160(&self) -> Vec<u8> {
		let mut hasher = Ripemd160::new();
		hasher.input(self);
		let mut res = vec![0u8; 20];
		hasher.result(&mut res);
		res
	}

	fn sha256_ripemd160(&self) -> Vec<u8> {
		let mut sha256 = Sha256::new();
		sha256.input(self);
		let mut res = vec![0u8; 32];
		sha256.result(&mut res);
		let mut hasher = Ripemd160::new();
		hasher.input(&res);
		let mut res = vec![0u8; 20];
		hasher.result(&mut res);
		res
	}

	fn hmac_sha512(&self, key: &[u8]) -> Vec<u8> {
		let mut hmac = Hmac::new(Sha512::new(), key);

		hmac.input(self);
		let res = hmac.result();
		res.code().to_vec()
	}
}

fn hex_encode(bytes: &[u8]) -> String {
	hex::encode(bytes)
}

trait HashableForString {
	fn hash256(&self) -> String;
	fn ripemd160(&self) -> String;
	fn sha256_ripemd160(&self) -> String;
	fn hmac_sha512(&self, key: &str) -> String;
	fn hash160(&self) -> String;
}
impl HashableForString for String {
	fn hash256(&self) -> String {
		hex_encode(&self.as_bytes().hash256())
	}

	fn ripemd160(&self) -> String {
		hex_encode(&self.as_bytes().ripemd160())
	}

	fn sha256_ripemd160(&self) -> String {
		hex_encode(&self.as_bytes().sha256_ripemd160())
	}

	fn hmac_sha512(&self, key: &str) -> String {
		hex_encode(&self.as_bytes().hmac_sha512(key.as_bytes()))
	}

	fn hash160(&self) -> String {
		let hash = self.as_bytes().sha256_ripemd160();
		bs58::encode(&hash[..]).into_string()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_hash256_for_bytes() {
		let data = b"hello world";
		let expected = "7509e5bda0c762d2bac7f90d758b5b2263fa01ccbc542ab5e3df163be08e6ca9";
		let result = data.hash256();
		assert_eq!(hex_encode(&result), expected);
	}

	#[test]
	fn test_hash256_for_string() {
		let data = String::from("hello world");
		let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
		assert_eq!(data.hash256(), expected);
	}

	#[test]
	fn test_ripemd160_for_bytes() {
		let data = b"hello world";
		// Use the expected hash value for "hello world" using RIPEMD160
		let expected = "98c615784ccb5fe5936fbc0cbe9dfdb408d92f0f";
		let result = data.ripemd160();
		assert_eq!(hex_encode(&result), expected);
	}

	#[test]
	fn test_ripemd160_for_string() {
		let data = String::from("hello world");
		let expected = "98c615784ccb5fe5936fbc0cbe9dfdb408d92f0f";
		assert_eq!(data.ripemd160(), expected);
	}

	#[test]
	fn test_sha256_ripemd160_for_bytes() {
		let data = b"hello world";
		// Use the expected hash value for "hello world" using SHA256 followed by RIPEMD160
		let expected = "..."; // fill this in
		let result = data.sha256_ripemd160();
		assert_eq!(hex_encode(&result), expected);
	}

	#[test]
	fn test_sha256_ripemd160_for_string() {
		let data = String::from("hello world");
		let expected = "..."; // fill this in
		assert_eq!(data.sha256_ripemd160(), expected);
	}

	#[test]
	fn test_hmac_sha512_for_bytes() {
		let data = b"hello world";
		let key = b"secret";
		// Use the expected HMAC-SHA512 value for "hello world" with key "secret"
		let expected = "..."; // fill this in
		let result = data.hmac_sha512(key);
		assert_eq!(hex_encode(&result), expected);
	}

	#[test]
	fn test_hmac_sha512_for_string() {
		let data = String::from("hello world");
		let key = "secret";
		let expected = "..."; // fill this in
		assert_eq!(data.hmac_sha512(key), expected);
	}

	#[test]
	fn test_hash160_for_string() {
		let data = String::from("hello world");
		// Use the expected hash value for "hello world" using SHA256 followed by RIPEMD160 and then base58 encoded
		let expected = "..."; // fill this in
		assert_eq!(data.hash160(), expected);
	}
}
