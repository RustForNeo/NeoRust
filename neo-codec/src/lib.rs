mod binary_decoder;
pub mod binary_encoder;
mod error;

pub use binary_decoder::*;
pub use binary_encoder::*;
pub use error::*;

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
