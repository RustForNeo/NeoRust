// ScriptBuilder.rs

use std::collections::BTreeMap;
use p256::pkcs8::der::Encode;
use crate::script::{op_code, interop_service};
use crate::script::interop_service::InteropService;
use crate::script::op_code::OpCode;
use crate::types::Bytes;
use crate::types::contract_parameter::{ContractParameter, ParameterValue};

pub struct ScriptBuilder {
    bytes: Vec<u8>,
}

impl ScriptBuilder {

    pub fn new() -> Self {
        ScriptBuilder { bytes: vec![] }
    }

    pub fn op_codes(&mut self, op_codes: &[OpCode]) -> &mut Self {
        for &op in op_codes {
            self.bytes.push(op as u8);
        }
        self
    }

    pub fn op_code(&mut self, op_code: OpCode, argument: &[u8]) -> &mut Self {
        self.bytes.push(op_code as u8);
        self.bytes.extend_from_slice(argument);
        self
    }

    pub fn contract_call(&mut self, hash160: &[u8; 20], method: &str, params: &[Option<ContractParameter>], max_items: usize) -> Result<&mut Self, ()> {
        if params.is_empty() {
            self.op_codes(&[OpCode::NewArray])?;
        } else {
            self.push_params(params)?;
        }
        Ok(self.push_data(method.as_bytes())
            .push_data(hash160)
            .sys_call(InteropService::SystemContractCall))
    }

    pub fn sys_call(&mut self, operation: InteropService) -> &mut Self {
        self.op_codes(&[OpCode::Syscall])
            .push_data(&operation.hash().to_vec().unwrap())
    }

    pub fn push_params(&mut self, params: &[Option<ContractParameter>]) -> Result<&mut Self, ()> {
        for param in params {
            self.push_param(param)?;
        }
        Ok(self.push_int(params.len() as i64)?
            .op_codes(&[OpCode::Pack]))
    }

    pub fn push_param(&mut self, param: &Option<ContractParameter>) -> Result<&mut Self, ()> {
        match param {
            None => {
                self.op_codes(&[OpCode::PushNull]);
                Ok(self)
            }
            Some(param) => {
                match &param.value {
                    Some(ContractParameterValue::Bool(b)) => self.push_bool(*b),
                    Some(ContractParameterValue::Integer(i)) => self.push_int(*i),
                    Some(ContractParameterValue::ByteArray(b)) => self.push_data(b),
                    Some(ContractParameterValue::String(s)) => self.push_data(s.as_bytes()),
                    Some(ContractParameterValue::Array(a)) => self.push_array(a),
                    Some(ContractParameterValue::Map(m)) => self.push_map(m),
                    None => Ok(self),
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn push_int(&mut self, n: i64) -> Result<&mut Self, ()> {
        if n >= 0 && n <= 16 {
            self.op_codes(&[OpCode::Push0 + n as u8]);
            Ok(self)
        } else {
            let bytes = n.to_be_bytes();
            match bytes.len() {
                1 => self.op_code(OpCode::PushInt8, &bytes[..]),
                2 => self.op_code(OpCode::PushInt16, &bytes[..]),
                4 => self.op_code(OpCode::PushInt32, &bytes[..]),
                8 => self.op_code(OpCode::PushInt64, &bytes[..]),
                _ => Err(()),
            }
        }
    }

    fn push_bool(&mut self, b: bool) -> &mut Self {
        if b {
            self.op_codes(&[OpCode::Push1]);
        } else {
            self.op_codes(&[OpCode::Push0]);
        }
        self
    }

    pub(crate) fn push_data(&mut self, data: &[u8]) -> &mut Self {
        match data.len() {
            0...255 => {
                self.op_codes(&[OpCode::PushData1]);
                self.bytes.push(data.len() as u8);
            }
            0...65535 => {
                self.op_codes(&[OpCode::PushData2]);
                self.bytes.extend_from_slice(&(data.len() as u16).to_be_bytes());
            }
            _ => {
                self.op_codes(&[OpCode::PushData4]);
                self.bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
            }
        }
        self.bytes.extend_from_slice(data);
        self
    }

    fn push_array(&mut self, params: &[ContractParameter]) -> Result<&mut Self, ()> {
        if params.is_empty() {
            self.op_codes(&[OpCode::NewArray]);
            Ok(self)
        } else {
            self.push_params(params.iter().filter_map(|p| Some(p)).collect())
        }
    }

    fn push_map(&mut self, map: &BTreeMap<ContractParameter, ContractParameter>) -> Result<&mut Self, ()> {
        for (k, v) in map {
            self.push_param(&Some(v.clone().into()))?;
            self.push_param(&Some(k.clone().into()))?;
        }
        Ok(self.push_int(map.len() as i64)?
            .op_codes(&[OpCode::Pack]))
    }

    pub fn pack(&mut self) -> &mut Self {
        self.op_codes(&[OpCode::Pack]);
        self
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

}

pub enum ContractParameterValue {
    Bool(bool),
    Integer(i64),
    ByteArray(Vec<u8>),
    String(String),
    Array(Vec<ContractParameter>),
    Map(BTreeMap<ContractParameter, ContractParameter>),
}