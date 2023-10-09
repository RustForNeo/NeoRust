use p256::{
	ecdsa::{
		signature::Verifier, Error as P256SignatureError, Signature as P256Signature, VerifyingKey,
	},
	PublicKey as P256PublicKey,
};
use primitive_types::{H160, U256};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, str::FromStr};
use thiserror::Error;

type Address = H160;
// type U32 = u32;

/// An error involving a signature.
#[derive(Debug, Error)]
pub enum SignatureError {
	/// Invalid length, secp256k1 signatures are 65 bytes
	#[error("invalid signature length, got {0}, expected 65")]
	InvalidLength(usize),
	/// When parsing a signature from string to hex
	#[error(transparent)]
	DecodingError(#[from] hex::FromHexError),
	/// Thrown when signature verification failed (i.e. when the address that
	/// produced the signature did not match the expected address)
	#[error("Signature verification failed. Expected {0}, got {1}")]
	VerificationError(Address, Address),
	/// Internal error during signature recovery
	#[error(transparent)]
	P256Error(#[from] P256SignatureError),
	/// Error in recovering public key from signature
	#[error("Public key recovery error")]
	RecoveryError,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy, Hash)]
/// An ECDSA signature
pub struct Signature {
	/// R value
	pub r: U256,
	/// S Value
	pub s: U256,
	/// V value
	pub v: u64,
}

impl fmt::Display for Signature {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let signature_bytes: [u8; 65] = self.into();
		write!(f, "{:x?}", signature_bytes)
	}
}

impl Signature {
	/// Verifies that signature on `message` was produced by `address`
	fn verify<M>(&self, message: M, public_key: &P256PublicKey) -> Result<(), SignatureError>
	where
		M: AsRef<[u8]>,
	{
		let mut msg = [0u8; 32];
		msg.copy_from_slice(message.as_ref());

		let r_bytes: [u8; 32] = self.r.into();
		let s_bytes: [u8; 32] = self.s.into();
		let signature = P256Signature::from_scalars(r_bytes, s_bytes)
			.map_err(|e| SignatureError::P256Error(e))?;

		let verifying_key = VerifyingKey::from(public_key);
		verifying_key
			.verify(&msg, &signature)
			.map_err(|_| SignatureError::VerificationError);

		Ok(())
	}

	/// Retrieves the recovery signature.
	fn as_signature(&self) -> Result<P256Signature, SignatureError> {
		let r_bytes: [u8; 32] = self.r.into();
		let s_bytes: [u8; 32] = self.s.into();
		P256Signature::from_scalars(r_bytes, s_bytes).map_err(|e| SignatureError::P256Error(e))
	}

	/// Copies and serializes `self` into a new `Vec` with the recovery id included
	#[allow(clippy::wrong_self_convention)]
	pub fn to_vec(&self) -> Vec<u8> {
		self.into()
	}

	pub fn decode_signature(buf: &mut &[u8]) -> Result<Self, SignatureError> {
		if buf.len() < 65 {
			return Err(SignatureError::InvalidLength(buf.len()))
		}

		let r = U256::from_big_endian(&buf[0..32]);
		let s = U256::from_big_endian(&buf[32..64]);
		let v = u64::from_be_bytes([
			buf[64], buf[65], buf[66], buf[67], buf[68], buf[69], buf[70], buf[71],
		]);

		Ok(Self { r, s, v })
	}
}

impl<'a> TryFrom<&'a [u8]> for Signature {
	type Error = SignatureError;

	fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
		if bytes.len() != 65 {
			return Err(SignatureError::InvalidLength(bytes.len()))
		}

		let v = bytes[64];
		let r = U256::from_big_endian(&bytes[0..32]);
		let s = U256::from_big_endian(&bytes[32..64]);

		Ok(Signature { r, s, v: v.into() })
	}
}

impl From<&Signature> for [u8; 65] {
	fn from(src: &Signature) -> [u8; 65] {
		let mut sig = [0u8; 65];
		let mut r_bytes = [0u8; 32];
		let mut s_bytes = [0u8; 32];
		src.r.to_big_endian(&mut r_bytes);
		src.s.to_big_endian(&mut s_bytes);
		sig[..32].copy_from_slice(&r_bytes);
		sig[32..64].copy_from_slice(&s_bytes);
		sig[64] = src.v as u8;
		sig
	}
}

impl FromStr for Signature {
	type Err = SignatureError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let bytes = hex::decode(s).map_err(SignatureError::DecodingError)?;
		Signature::try_from(&bytes[..])
	}
}

impl From<Signature> for Vec<u8> {
	fn from(src: Signature) -> Vec<u8> {
		<[u8; 65]>::from(src).to_vec()
	}
}

impl From<Signature> for [u8; 65] {
	fn from(src: Signature) -> [u8; 65] {
		let r_bytes: [u8; 32] = src.r.into();
		let s_bytes: [u8; 32] = src.s.into();
		let mut sig = [0u8; 65];
		sig[..32].copy_from_slice(&r_bytes);
		sig[32..64].copy_from_slice(&s_bytes);
		sig[64] = src.v as u8;
		sig
	}
}

impl From<&Signature> for Vec<u8> {
	fn from(src: &Signature) -> Vec<u8> {
		let sig_array: [u8; 65] = src.into();
		sig_array.to_vec()
	}
}

impl open_fastrlp::Decodable for Signature {
	fn decode(buf: &mut &[u8]) -> Result<Self, open_fastrlp::DecodeError> {
		match Self::decode_signature(buf) {
			Ok(sig) => Ok(sig),
			Err(SignatureError::InvalidLength(_)) => Err(open_fastrlp::DecodeError::InputTooShort),
			Err(SignatureError::DecodingError(_)) =>
				Err(open_fastrlp::DecodeError::Custom(&"DecodingError")),
			Err(SignatureError::VerificationError(_, _)) =>
				Err(open_fastrlp::DecodeError::Custom(&"VerificationError")),
			Err(SignatureError::P256Error(_)) =>
				Err(open_fastrlp::DecodeError::Custom(&"P256Error")),
			_ => Err(open_fastrlp::DecodeError::Custom(&"UnknownError")),
		}
	}
}

impl open_fastrlp::Encodable for Signature {
	fn length(&self) -> usize {
		// This assumes U256 and u64 have a length() method
		32 + 32 + 8 // r's length + s's length + v's length
	}

	fn encode(&self, out: &mut dyn bytes::BufMut) {
		let r_bytes: [u8; 32] = self.r.into();
		let s_bytes: [u8; 32] = self.s.into();
		out.put_slice(&r_bytes);
		out.put_slice(&s_bytes);
		out.put_u64(self.v);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn signature_from_str() {
		let s1 = Signature::from_str(
            "0xaa231fbe0ed2b5418e6ba7c19bee2522852955ec50996c02a2fe3e71d30ddaf1645baf4823fea7cb4fcc7150842493847cfb6a6d63ab93e8ee928ee3f61f503500"
        ).expect("could not parse 0x-prefixed signature");

		let s2 = Signature::from_str(
            "aa231fbe0ed2b5418e6ba7c19bee2522852955ec50996c02a2fe3e71d30ddaf1645baf4823fea7cb4fcc7150842493847cfb6a6d63ab93e8ee928ee3f61f503500"
        ).expect("could not parse non-prefixed signature");

		assert_eq!(s1, s2);
	}
}
