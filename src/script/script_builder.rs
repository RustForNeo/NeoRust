// ScriptBuilder.rs

use crate::script::{op_code, interop_service};
use crate::script::interop_service::InteropService;
use crate::script::op_code::OpCode;
use crate::types::Bytes;

pub struct ScriptBuilder {
    script: Bytes,
}

impl ScriptBuilder {

    pub fn new() -> Self {
        Self { script: Bytes::new() }
    }

    pub fn push_data(&mut self, data: Bytes) {
        // push data to script
    }

    pub fn op_code(&mut self, op_code: OpCode) {
        // add op code
    }

    pub fn sys_call(&mut self, service: InteropService) {
        // add sys call
    }

    pub fn build(&self) -> Bytes {
        self.script.clone()
    }

}