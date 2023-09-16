// invocation_script


use p256::ecdsa::Signature;
use crate::crypto::key_pair::KeyPair;
use crate::crypto::sign::SignatureData;
use crate::script::script_builder::ScriptBuilder;
use crate::types::Bytes;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InvocationScript {
    script: Bytes,
}

impl InvocationScript {
    pub fn new() -> Self {
        Self { script: Bytes::new() }
    }

    pub fn from(script: Bytes) -> Self {
        Self { script }
    }

    pub fn from_signature(signature: &SignatureData) -> Self {
        let mut builder = ScriptBuilder::new();
        builder.push_data(&signature.concatenated());
        Self { script: builder.into_bytes() }
    }

    pub fn from_message_and_key_pair(message: Bytes, key_pair: &KeyPair) -> Result<Self, ()> {
        let signature = Sign::sign_message(&message, key_pair)?;
        let mut builder = ScriptBuilder::new();
        builder.push_data(&signature.concatenated());
        Ok(Self { script: builder.into_bytes() })
    }

    pub fn from_signatures(signatures: &[SignatureData]) -> Self {
        let mut builder = ScriptBuilder::new();
        for signature in signatures {
            builder.push_data(&signature.concatenated());
        }
        Self { script: builder.into_bytes() }
    }

    // other methods
}