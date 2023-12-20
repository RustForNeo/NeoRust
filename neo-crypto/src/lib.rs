pub mod base58_helper;
pub mod error;
pub mod hash;
pub mod key_pair;
pub mod keys;
pub mod nep2;
pub mod sign;
pub mod signature;
pub mod wif;
pub fn add(left: usize, right: usize) -> usize {
	left + right
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let result = add(2, 2);
		assert_eq!(result, 4);
	}
}
