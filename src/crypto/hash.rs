use crypto::{hmac::Hmac, ripemd160::Ripemd160, sha2::Sha512};
use sha2::{Digest, Sha256};

pub trait HashableForVec {
	fn hash256(&self) -> Vec<u8>;
	fn ripemd160(&self) -> Vec<u8>;
	fn sha256_ripemd160(&self) -> Vec<u8>;
	fn hmac_sha512(&self, key: &[u8]) -> Vec<u8>;
}

impl HashableForVec for [u8] {
	fn hash256(&self) -> Vec<u8> {
		let mut hasher = Sha256::new();
		hasher.update(self);
		hasher.finalize().into_bytes().to_vec()
	}

	fn ripemd160(&self) -> Vec<u8> {
		let mut hasher = Ripemd160::new();
		hasher.update(self);
		hasher.finalize().into_bytes().to_vec()
	}

	fn sha256_ripemd160(&self) -> Vec<u8> {
		let mut sha256 = Sha256::new();
		sha256.update(self);
		let hash = sha256.finalize();

		let mut ripemd160 = Ripemd160::new();
		ripemd160.update(&hash);
		ripemd160.finalize().into_bytes().to_vec()
	}

	fn hmac_sha512(&self, key: &[u8]) -> Vec<u8> {
		let mut mac = Hmac::<Sha512>::new_varkey(key).expect("HMAC accepts keys of any size");
		mac.update(self);
		mac.finalize().into_bytes().to_vec()
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
