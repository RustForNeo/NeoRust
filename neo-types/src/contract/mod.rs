use primitive_types::H256;
use sha2::{Digest, Sha256};

pub mod contract_manifest;
pub mod contract_method_token;
pub mod contract_nef;
pub mod contract_parameter;
pub mod contract_parameter_type;
pub mod contract_state;
pub mod contract_storage_entry;
pub mod invocation_result;
pub mod native_contract_state;
pub mod nef_file;
pub mod nep17contract;

pub fn hash_message(message: &[u8]) -> H256 {
	let mut hasher = Sha256::new();
	hasher.update(message);
	let result = hasher.finalize();
	H256::from_slice(&result)
}
