use p256::ecdsa::Signature;
use primitive_types::H160;
use crate::neo_error::NeoError;
use crate::script::interop_service::InteropService;
use crate::script::op_code::OpCode;
use crate::script::script_builder::ScriptBuilder;
use crate::serialization::binary_reader::BinaryReader;
use crate::types::{Bytes, ECPublicKey};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VerificationScript {
    script: Bytes,
}

impl VerificationScript {

    pub fn new(script: Bytes) -> Self {
        Self { script }
    }

    pub fn from_public_key(public_key: &ECPublicKey) -> Self {
        let mut builder = ScriptBuilder::new();
        builder.push_data(public_key.to_bytes())
            .OpCode(OpCode::Syscall)
            .push_data(InteropService::SystemCryptoCheckSig.hash().as_bytes());
        Self::new(builder.build())
    }

    pub fn from_multisig(public_keys: &[ECPublicKey], threshold: u8) -> Self {
        // Build multi-sig script
        let mut builder = ScriptBuilder::new();
        builder.push_int(threshold as i64).expect("Threshold must be between 1 and 16");
        for key in public_keys {
            builder.push_data(key.to_bytes()).expect("TODO: panic message");
        }
        builder.push_int(public_keys.len() as i64)
            .OpCode(OpCode::Syscall)
            .push_data( InteropService::SystemCryptoCheckMultisig.hash().as_bytes());
        Self::new(builder.build())
    }

    pub fn is_single_sig(&self) -> bool {
        self.script.len() == 35
            && self.script[0] == OpCode::PushData1 as u8
            && self.script[34] == OpCode::Syscall as u8
    }

    pub fn is_multisig(&self) -> bool {
        if self.script.len() < 37 {
            return false;
        }

        let mut reader = BinaryReader::new(&self.script);

        let n = reader.read_var_int().unwrap();
        if !(1..16).contains(&n) {
            return false;
        }

        let mut m = 0;
        while reader.read_u8() == Some(OpCode::PushData1 as u8) {
            let len = reader.read_u8().unwrap();
            if len != 33 {
                return false;
            }
            let _ = reader.skip(33);
            m += 1;
        }

        if !(m >= n && m <= 16) {
            return false;
        }

        // additional checks
        let service_bytes = &self.script[self.script.len()-4..];
        if service_bytes != &InteropService::SystemCryptoCheckMultisig.hash().into_bytes() {
            return false;
        }

        if m != reader.read_var_int().unwrap() {
            return false;
        }

        if reader.read_u8() != Some(OpCode::Syscall as u8) {
            return false;
        }

        true
    }

    // other methods
    pub fn hash(&self) -> H160 {
        H160::from_data(&self.script)
    }

    pub fn get_signatures(&self) -> Vec<Signature> {
        let mut reader = BinaryReader::new(&self.script);
        let mut signatures = vec![];

        while reader.read_u8() == Some(OpCode::PushData1 as u8) {
            let len = reader.read_u8().unwrap();
            let sig = Signature::from_slice(&reader.read_bytes(len as usize).unwrap());
            signatures.push(sig);
        }

        signatures
    }

    pub fn get_public_keys(&self) -> Result<Vec<ECPublicKey>, NeoError> {
        if self.is_single_sig() {
            let mut reader = BinaryReader::new(&self.script);
            reader.read_u8(); // skip pushdata1
            reader.read_u8(); // skip length

            let mut point = [0; 33];
            point.copy_from_slice(&reader.read_bytes(33).unwrap());

            let key = ECPublicKey::from_bytes(&point)?;
            return Ok(vec![key]);
        }

        if self.is_multisig() {
            let mut reader = BinaryReader::new(&self.script);
            reader.read_var_int().unwrap(); // skip threshold

            let mut keys = vec![];
            while reader.read_u8() == Some(OpCode::PushData1 as u8) {
                reader.read_u8(); // skip length
                let mut point = [0; 33];
                point.copy_from_slice(&reader.read_bytes(33).unwrap());
                keys.push(ECPublicKey::from_bytes(&point)?);
            }

            Ok(keys)
        }

        Err(NeoError::InvalidScript("Invalid verification script".to_string()))
    }
}