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
