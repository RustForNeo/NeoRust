use crate::error::CryptoError;
use p256::{elliptic_curve::sec1::FromEncodedPoint, EncodedPoint, PublicKey, SecretKey};
use primitive_types::H160;

pub trait PrivateKeyExtension
where
	Self: Sized,
{
	fn to_vec(&self) -> Vec<u8>;

	fn from_slice(slice: &[u8]) -> Result<Self, CryptoError>;
}

impl PrivateKeyExtension for SecretKey {
	fn to_vec(&self) -> Vec<u8> {
		self.to_bytes().to_vec()
	}

	fn from_slice(slice: &[u8]) -> Result<Self, CryptoError> {
		if slice.len() != 32 {
			return Err(CryptoError::InvalidPublicKey)
		}

		let mut arr = [0u8; 32];
		arr.copy_from_slice(slice);
		Ok(Self::from_bytes(&arr.into())
			.map_err(|_| CryptoError::InvalidPublicKey)
			.unwrap())
	}
}

pub trait PublicKeyExtension
where
	Self: Sized,
{
	fn to_vec(&self) -> Vec<u8>;
	fn from_slice(slice: &[u8]) -> Result<Self, CryptoError>;
}
impl PublicKeyExtension for PublicKey {
	fn to_vec(&self) -> Vec<u8> {
		self.to_encoded_point(false).as_bytes().to_vec()
	}
	fn from_slice(slice: &[u8]) -> Result<Self, CryptoError> {
		if slice.len() != 65 && slice.len() != 33 {
			return Err(CryptoError::InvalidPublicKey)
		}
		Ok(Self::from_encoded_point(&EncodedPoint::from_bytes(&slice).unwrap()).unwrap())
	}
}
