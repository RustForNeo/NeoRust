// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]
#![allow(dead_code)]

use serde::{
	de::Deserializer,
	ser::{SerializeStruct, Serializer},
	Deserialize, Serialize,
};

use blake2::digest::{Update, VariableOutput};
use tiny_keccak::{Hasher, Keccak};

use rand::{
	rngs::StdRng,
	seq::{IteratorRandom, SliceRandom},
	SeedableRng,
};

use crate::types::{Address, PublicKey};
use futures::AsyncWriteExt;
use p256::{elliptic_curve::bigint::U64, U256};
use primitive_types::{H160, H256};
use std::collections::HashMap;

pub fn keccak_hash(msg: &[u8]) -> [u8; 32] {
	// hash the message with keccak-256
	let mut keccak = Keccak::v256();
	let mut msg_hash = [0_u8; 32];
	keccak.update(msg);
	keccak.finalize(&mut msg_hash);
	msg_hash
}

// pub fn blake2_hash(msg: &[u8]) -> [u8; 32] {
//     // create a Blake2b object
//     let mut hasher = VarBlake2b::new(32).unwrap();
//
//     hasher.update(&msg);
//
//     let mut res = [0_u8; 32];
//     // read hash digest and consume hasher
//     hasher.finalize_variable(|result| {
//         res.copy_from_slice(result);
//     });
//     res
// }

pub fn parse_string_u64(u64_str: &str) -> U64 {
	if u64_str.starts_with("0x") {
		U64::from_str_radix(u64_str, 16).unwrap()
	} else {
		U64::from_str_radix(u64_str, 10).unwrap()
	}
}

pub fn parse_string_u256(u256_str: &str) -> U256 {
	if u256_str.starts_with("0x") {
		U256::from_str_radix(u256_str, 16).unwrap()
	} else {
		U256::from_str_radix(u256_str, 10).unwrap()
	}
}

pub fn parse_string_h160(h160_str: &str) -> H160 {
	let bytes = hex::decode(h160_str.trim_start_matches("0x")).unwrap();
	let mut padded_bytes = [0_u8; 20];
	padded_bytes[20 - bytes.len()..].copy_from_slice(&bytes);
	H160::from_slice(&padded_bytes)
}

pub fn encode_string_h160(h160: &H160) -> String {
	// here we just make use of the display functionality of H160
	// the debug string prints in full form (hex)
	format!("{:?}", h160).to_owned()
}

pub fn parse_string_h256(h256_str: &str) -> H256 {
	// hex string can be one of two forms
	// 1. 0x1123a5
	// 2.   1123a5
	// NOTE: for ethereum h256, the bytestring is represented in "big-endian" form
	// that is for an array of the form
	//   lsb [a5, 23, 11] msb
	// index: 0   1   2
	// the corresponding bytestring is of the form:
	// 0xa523110000..00
	//
	// Here, we'll strip the initial 0x and parse it using hex::decode
	// which gives us the exact representation we want.
	// 0xa5 23 11 00 .. 00
	//   a5 23 11 00 .. 00
	//  [a5,23,11,00,..,00] <- in the right endianness

	let bytes = hex::decode(h256_str.trim_start_matches("0x")).unwrap();
	// pad the bytes to 32bytes
	let mut padded_bytes = [0_u8; 32];
	padded_bytes[32 - bytes.len()..].copy_from_slice(&bytes);

	H256::from_slice(&padded_bytes)
}

pub fn encode_string_h256(h256: &H256) -> String {
	// here we just make use of the display functionality of H256
	// the debug string prints in full form (hex)
	format!("{:?}", h256).to_owned()
}

pub fn encode_string_u256(u256: &U256) -> String {
	format!("0x{:x}", u256).to_owned()
}

pub fn encode_vec_string_vec_u256(item: Vec<U256>) -> Vec<String> {
	item.iter().map(|x| encode_string_u256(&x)).collect()
}

pub fn parse_vec_string_vec_u256(item: Vec<String>) -> Vec<U256> {
	item.iter().map(|x| parse_string_u256(&x)).collect()
}

pub fn h256_to_u256(item: H256) -> U256 {
	U256::from_big_endian(item.as_bytes())
}

pub fn eth_secp256k1_to_accountid(pubkey: &PublicKey) -> Address {
	let mut data = [0_u8; 64];
	let mut key = pubkey.to_encoded_point(false).to_bytes();
	data.copy_from_slice(key.as_ref());

	let mut keccak = Keccak::v256();
	let mut msg_hash = [0_u8; 32];
	keccak.update(&data);
	keccak.finalize(&mut msg_hash);
	let mut addr_bytes = [0_u8; 20];
	addr_bytes.copy_from_slice(&msg_hash[12..]);
	addr_bytes.into()
}

pub fn bytes_to_string(mybytes: &[u8]) -> String {
	format!("0x{}", hex::encode(mybytes))
}

pub fn string_to_bytes(mystring: &str) -> Option<Vec<u8>> {
	if mystring.starts_with("0x") {
		let mystring = mystring.trim_start_matches("0x");
		let mybytes = match hex::decode(mystring) {
			Ok(mybytes) => Some(mybytes),
			Err(_) => None,
		};
		mybytes
	} else {
		None
	}
}

pub fn u256_sqrt(input: &U256) -> U256 {
	if *input < 2.into() {
		return input.clone()
	}
	let mut x: U256 = (input + U256::one()) >> 1;
	let mut y = input.clone();
	while x < y {
		y = x;
		x = (input / x + x) >> 1;
	}
	y
}

pub fn u256_min(x: U256, y: U256) -> U256 {
	if x > y {
		y
	} else {
		x
	}
}

#[cfg(test)]
mod test {
	use super::*;

	// #[test]
	// pub fn test_blake2var_hash() {
	//     let mut data = [0_u8; 24];
	//     data[0..4].copy_from_slice(b"evm:");
	//     data[4..24].copy_from_slice(&hex::decode("7EF99B0E5bEb8ae42DbF126B40b87410a440a32a").unwrap());
	//     let hash = blake2_hash(&data);
	//     let actual = hex::decode("65f5fbd10250447019bb8b9e06f6918d033b2feb6478470137b1a552656e2911").unwrap();
	//     assert_eq!(&hash, actual.as_slice());
	// }

	#[test]
	pub fn test_bytes_to_string() {
		let mybytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
		let bytestring = bytes_to_string(&mybytes);
		let orig_bytestring = "0x0102030405060708090a";
		assert_eq!(&bytestring, orig_bytestring);
		let error_bytestring = "0102030405060708090a";
		let error_mybytes = string_to_bytes(error_bytestring);
		assert_eq!(None, error_mybytes);
		let ok_mybytes = string_to_bytes(orig_bytestring).unwrap();
		assert_eq!(&mybytes[..], &ok_mybytes[..]);
	}
}
