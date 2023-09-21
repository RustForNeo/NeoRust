#![allow(unused_imports)]
#![allow(dead_code)]

#[cfg(feature = "substrate")]
use serde_big_array_substrate::big_array;
#[cfg(feature = "substrate")]
use serde_substrate as serde;

use crate::utils::*;
use hex;
use p256::{elliptic_curve::bigint::U64, U256};
use primitive_types::H256;
use reqwest::Url;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
	collections::{HashMap, HashSet},
	convert::TryInto,
};
use tiny_keccak::{Hasher, Keccak};

use crate::contract::nef_file::MethodToken;
use serde::ser::{SerializeMap, SerializeSeq};

use crate::{types::Address, utils::util::*};

pub fn serialize_bytes<S>(item: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let item_str = format!("0x{}", hex::encode(item));
	serializer.serialize_str(&item_str)
}

pub fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: String = Deserialize::deserialize(deserializer)?;
	let bytes = hex::decode(s.trim_start_matches("0x")).unwrap();
	Ok(bytes)
}

pub fn serialize_url<S>(item: Url, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	// deserialize_address
	let item_str = format!("{}", item);
	serializer.serialize_str(&item_str)
}

pub fn deserialize_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
	D: Deserializer<'de>,
{
	let s: String = Deserialize::deserialize(deserializer)?;
	let url = Url::parse(&s).unwrap();
	Ok(url)
}

pub fn serialize_u256<S>(item: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let item_str = format!("{}", item);
	serializer.serialize_str(&item_str)
}

pub fn deserialize_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
	D: Deserializer<'de>,
{
	let s: String = Deserialize::deserialize(deserializer)?;
	Ok(parse_string_u256(&s))
}

pub fn serialize_u32<S>(item: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let item_str = format!("0x{:x}", item);
	serializer.serialize_str(&item_str)
}

pub fn deserialize_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
	D: Deserializer<'de>,
{
	let s: String = Deserialize::deserialize(deserializer)?;
	let v = if s.starts_with("0x") {
		let s = s.trim_start_matches("0x");
		u32::from_str_radix(&s, 16).unwrap()
	} else {
		u32::from_str_radix(&s, 10).unwrap()
	};
	Ok(v)
}

pub fn serialize_u64<S>(item: &U64, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let item_str = format!("{}", item);
	serializer.serialize_str(&item_str)
}

pub fn deserialize_u64<'de, D>(deserializer: D) -> Result<U64, D::Error>
where
	D: Deserializer<'de>,
{
	let s: String = Deserialize::deserialize(deserializer)?;
	Ok(parse_string_u64(&s))
}

pub fn deserialize_address<'de, D>(deserializer: D) -> Result<Address, D::Error>
where
	D: Deserializer<'de>,
{
	let s: String = Deserialize::deserialize(deserializer)?;
	let addr = parse_string_h160(&s);
	Ok(addr)
}

pub fn serialize_address<S>(item: &Address, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let item_str = encode_string_h160(&item);
	serializer.serialize_str(&item_str)
}

pub fn deserialize_vec_address<'de, D>(deserializer: D) -> Result<Vec<Address>, D::Error>
where
	D: Deserializer<'de>,
{
	let string_seq = <Vec<String>>::deserialize(deserializer)?;
	let mut vec: Vec<Address> = Vec::new();
	for v_str in string_seq {
		let v = parse_string_h160(&v_str);
		vec.push(v);
	}
	Ok(vec)
}

pub fn serialize_vec_address<S>(item: &Vec<Address>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let mut seq = serializer.serialize_seq(Some(item.len()))?;
	for i in item {
		seq.serialize_element(&encode_string_h160(i))?;
	}
	seq.end()
}

pub fn serialize_vec_methodtoken<S>(
	item: &Vec<MethodToken>,
	serializer: S,
) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let mut seq = serializer.serialize_seq(Some(item.len()))?;
	for i in item {
		seq.serialize_element(&i)?;
	}
	seq.end()
}

pub fn deserialize_vec_methodtoken<'de, D>(deserializer: D) -> Result<Vec<MethodToken>, D::Error>
where
	D: Deserializer<'de>,
{
	let string_seq = <Vec<MethodToken>>::deserialize(deserializer)?;
	let mut vec: Vec<MethodToken> = Vec::new();
	for v_str in string_seq {
		let v = v_str;
		vec.push(v);
	}
	Ok(vec)
}

pub fn serialize_h256<S>(item: &H256, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_str(&encode_string_h256(item))
}

pub fn deserialize_h256<'de, D>(deserializer: D) -> Result<H256, D::Error>
where
	D: Deserializer<'de>,
{
	let s: String = Deserialize::deserialize(deserializer)?;
	Ok(parse_string_h256(&s))
}

pub fn serialize_hashset_u256<S>(item: &HashSet<U256>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let mut seq = serializer.serialize_seq(Some(item.len()))?;
	for i in item {
		seq.serialize_element(&encode_string_u256(i))?;
	}
	seq.end()
}

pub fn deserialize_hashset_u256<'de, D>(deserializer: D) -> Result<HashSet<U256>, D::Error>
where
	D: Deserializer<'de>,
{
	let string_seq = <HashSet<String>>::deserialize(deserializer)?;
	let mut hashset: HashSet<U256> = HashSet::new();
	for v_str in string_seq {
		let v = parse_string_u256(&v_str);
		hashset.insert(v);
	}
	Ok(hashset)
}

pub fn serialize_vec_h256<S>(item: &Vec<H256>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let mut seq = serializer.serialize_seq(Some(item.len()))?;
	for i in item {
		seq.serialize_element(&encode_string_h256(i))?;
	}
	seq.end()
}

pub fn deserialize_vec_h256<'de, D>(deserializer: D) -> Result<Vec<H256>, D::Error>
where
	D: Deserializer<'de>,
{
	let string_seq = <Vec<String>>::deserialize(deserializer)?;
	let mut vec: Vec<H256> = Vec::new();
	for v_str in string_seq {
		let v = parse_string_h256(&v_str);
		vec.push(v);
	}
	Ok(vec)
}

pub fn serialize_vec_u256<S>(item: &Vec<U256>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let mut seq = serializer.serialize_seq(Some(item.len()))?;
	for i in item {
		seq.serialize_element(&encode_string_u256(i))?;
	}
	seq.end()
}

pub fn deserialize_vec_u256<'de, D>(deserializer: D) -> Result<Vec<U256>, D::Error>
where
	D: Deserializer<'de>,
{
	let string_seq = <Vec<String>>::deserialize(deserializer)?;
	let mut vec: Vec<U256> = Vec::new();
	for v_str in string_seq {
		let v = parse_string_u256(&v_str);
		vec.push(v);
	}
	Ok(vec)
}

pub fn serialize_hashmap_u256_hashset_u256<S>(
	item: &HashMap<U256, HashSet<U256>>,
	serializer: S,
) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let mut map = serializer.serialize_map(Some(item.len()))?;
	for (k, v) in item {
		let value: HashSet<String> = v.iter().map(|x| encode_string_u256(&x)).collect();
		map.serialize_entry(&encode_string_u256(k), &value)?;
	}
	map.end()
}

pub fn deserialize_hashmap_u256_hashset_u256<'de, D>(
	deserializer: D,
) -> Result<HashMap<U256, HashSet<U256>>, D::Error>
where
	D: Deserializer<'de>,
{
	let map = <HashMap<String, HashSet<String>>>::deserialize(deserializer)?;
	let mut hashmap: HashMap<U256, HashSet<U256>> = HashMap::new();

	for (k, v) in map {
		let k_u256 = parse_string_u256(&k);
		let v_hashset_u256: HashSet<U256> = v.iter().map(|x| parse_string_u256(&x)).collect();
		hashmap.insert(k_u256, v_hashset_u256);
	}
	Ok(hashmap)
}

pub fn serialize_hashmap_address_u256<S>(
	item: &HashMap<Address, U256>,
	serializer: S,
) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let mut map = serializer.serialize_map(Some(item.len()))?;
	for (k, v) in item {
		map.serialize_entry(&encode_string_h160(k), &encode_string_u256(v))?;
	}
	map.end()
}

pub fn deserialize_hashmap_address_u256<'de, D>(
	deserializer: D,
) -> Result<HashMap<Address, U256>, D::Error>
where
	D: Deserializer<'de>,
{
	let map = <HashMap<String, String>>::deserialize(deserializer)?;
	let mut hashmap: HashMap<Address, U256> = HashMap::new();

	for (k, v) in map {
		let k_h160 = parse_string_h160(&k);
		let v_u256 = parse_string_u256(&v);
		hashmap.insert(k_h160, v_u256);
	}
	Ok(hashmap)
}

pub fn serialize_hashmap_u256_hashset_h256<S>(
	item: &HashMap<U256, HashSet<H256>>,
	serializer: S,
) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let mut map = serializer.serialize_map(Some(item.len()))?;
	for (k, v) in item {
		let value: HashSet<String> = v.iter().map(|x| encode_string_h256(&x)).collect();
		map.serialize_entry(&encode_string_u256(k), &value)?;
	}
	map.end()
}

pub fn deserialize_hashmap_u256_hashset_h256<'de, D>(
	deserializer: D,
) -> Result<HashMap<U256, HashSet<H256>>, D::Error>
where
	D: Deserializer<'de>,
{
	let map = <HashMap<String, HashSet<String>>>::deserialize(deserializer)?;
	let mut hashmap: HashMap<U256, HashSet<H256>> = HashMap::new();

	for (k, v) in map {
		let k_u256 = parse_string_u256(&k);
		let v_hashset_h256: HashSet<H256> = v.iter().map(|x| parse_string_h256(&x)).collect();
		hashmap.insert(k_u256, v_hashset_h256);
	}
	Ok(hashmap)
}

pub fn serialize_hashmap_u256_vec_u256<S>(
	item: &HashMap<U256, Vec<U256>>,
	serializer: S,
) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let mut map = serializer.serialize_map(Some(item.len()))?;
	for (k, v) in item {
		let value: Vec<String> = v.iter().map(|x| encode_string_u256(&x)).collect();
		map.serialize_entry(&encode_string_u256(k), &value)?;
	}
	map.end()
}

pub fn deserialize_hashmap_u256_vec_u256<'de, D>(
	deserializer: D,
) -> Result<HashMap<U256, Vec<U256>>, D::Error>
where
	D: Deserializer<'de>,
{
	let map = <HashMap<String, Vec<String>>>::deserialize(deserializer)?;
	let mut hashmap: HashMap<U256, Vec<U256>> = HashMap::new();

	for (k, v) in map {
		let k_u256 = parse_string_u256(&k);
		let v_vec_u256: Vec<U256> = v.iter().map(|x| parse_string_u256(&x)).collect();
		hashmap.insert(k_u256, v_vec_u256);
	}
	Ok(hashmap)
}

#[cfg(test)]
mod test {
	use super::*;

	#[derive(Clone, Default, Debug, Serialize, Deserialize)]
	struct TestStruct {
		#[serde(serialize_with = "serialize_hashset_u256")]
		#[serde(deserialize_with = "deserialize_hashset_u256")]
		value: HashSet<U256>,
	}

	#[derive(Clone, Default, Debug, Serialize)]
	struct TestStruct2 {
		#[serde(serialize_with = "serialize_hashmap_u256_hashset_u256")]
		value2: HashMap<U256, HashSet<U256>>,
	}

	#[test]
	fn test_serialize_hashset_u256() {
		let mut v: HashSet<U256> = HashSet::new();
		v.insert(10.into());
		v.insert(0x10000.into());
		let _copy = v.clone();
		let test_struct = TestStruct { value: v };
		let json_string = serde_json::to_string_pretty(&test_struct).unwrap();
		println!("{}", json_string);
		let v_copy: TestStruct = serde_json::from_str(&json_string).unwrap();
		assert_eq!(test_struct.value, v_copy.value);
	}

	#[test]
	fn test_serialize_hashmap_u256_hashset_u256() {
		let mut v: HashMap<U256, HashSet<U256>> = HashMap::new();
		let mut v2: HashSet<U256> = HashSet::new();
		v2.insert(10.into());
		v2.insert(0x10000.into());
		v.insert(20.into(), v2);
		let test_struct = TestStruct2 { value2: v };
		let json_string = serde_json::to_string_pretty(&test_struct).unwrap();
		println!("{}", json_string);
	}

	#[test]
	fn test_serialize_bytes() {
		#[derive(Clone, Default, Debug, Serialize, Deserialize)]
		struct TestStruct {
			#[serde(serialize_with = "serialize_bytes")]
			#[serde(deserialize_with = "deserialize_bytes")]
			value: Vec<u8>,
		}

		let v = TestStruct { value: vec![23, 253, 255, 255, 0, 123] };
		let json_string = serde_json::to_string_pretty(&v).unwrap();
		println!("{}", json_string);
		let v_copy: TestStruct = serde_json::from_str(&json_string).unwrap();
		assert_eq!(v.value, v_copy.value);
	}

	#[test]
	fn test_serialize_u32() {
		#[derive(Clone, Default, Debug, Serialize, Deserialize)]
		struct TestStruct {
			#[serde(serialize_with = "serialize_u32")]
			#[serde(deserialize_with = "deserialize_u32")]
			value: u32,
		}

		let v = TestStruct { value: 20 };
		let json_string = serde_json::to_string_pretty(&v).unwrap();
		println!("{}", json_string);
		let v_copy: TestStruct = serde_json::from_str(&json_string).unwrap();
		assert_eq!(v.value, v_copy.value);
	}

	#[test]
	fn test_serialize_vec_h256() {
		#[derive(Clone, Default, Debug, Serialize, Deserialize)]
		struct TestStruct {
			#[serde(serialize_with = "serialize_vec_h256")]
			#[serde(deserialize_with = "deserialize_vec_h256")]
			value: Vec<H256>,
		}

		let v = TestStruct {
			value: vec![parse_string_h256(
				"0x95ff99bcdac06fad4a141f06c5f9f1c65e71b188ff5978116a110c4170fd7355",
			)],
		};
		let json_string = serde_json::to_string_pretty(&v).unwrap();
		println!("{}", json_string);
		let v_copy: TestStruct = serde_json::from_str(&json_string).unwrap();
		assert_eq!(v.value, v_copy.value);
	}
}
