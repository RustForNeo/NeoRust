// nef_file

use serde::{Serialize, Deserialize};
use crate::types::Bytes;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NefFile {
    compiler: String,
    source_url: String,
    methods: Vec<MethodToken>,
    script: Bytes,
    checksum: Bytes,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MethodToken {
    hash: H160,
    method: String,
    params_count: u16,
    has_return: bool,
    call_flags: u8
}
// Implementations