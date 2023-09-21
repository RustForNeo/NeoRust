#![feature(const_trait_impl)]

extern crate core;
pub mod constant;
pub mod contract;
pub mod crypto;
pub mod neo_error;
pub mod protocol;
pub mod script;
pub mod serialization;
pub mod transaction;
pub mod types;
pub mod utils;
pub mod wallet;

pub use primitive_types::{H160, H256, U256};

pub fn add(left: usize, right: usize) -> usize {
	left + right
}

#[macro_use]
extern crate getset;

#[macro_use]
extern crate educe;

use crate::utils::*;

// #[macro_use]
// extern crate serde_with;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let result = add(2, 2);
		assert_eq!(result, 4);
	}
}
