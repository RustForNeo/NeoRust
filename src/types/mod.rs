use num_bigint::BigInt;
use p256::{PublicKey, SecretKey};

pub mod call_flags;
pub mod contract_parameter;
pub mod contract_parameter_type;
pub mod vm_state;
pub mod plugin_type;

// Bring EC types into scope

pub type ECPoint = p256::ecdsa::ECPoint;
pub type ECPrivateKey = SecretKey;
pub type ECPublicKey = PublicKey;

pub type Address = String;

pub type Byte = u8;
pub type Bytes = Vec<u8>;
