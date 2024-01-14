use crate::error::SignerError;
use neo_config::DEFAULT_ADDRESS_VERSION;
use neo_crypto::{
	hash::HashableForVec,
	keys::{PrivateKeyExtension, PublicKeyExtension, Secp256r1PrivateKey, Secp256r1PublicKey},
};
use neo_providers::core::script::script_builder::ScriptBuilder;
use neo_types::script_hash::{ScriptHash, ScriptHashExtension};
use primitive_types::H160;
use rustc_serialize::hex::ToHex;
use std::str::FromStr;
