// invocation_script


use p256::ecdsa::Signature;
use crate::crypto::key_pair::KeyPair;
use crate::script::script_builder::ScriptBuilder;
use crate::types::Bytes;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InvocationScript {
    script: Bytes,
}

impl InvocationScript {
    pub fn new(script: Bytes) -> Self {
        Self { script }
    }

    pub fn from_signature(signature: Signature) -> Self {
        let mut builder = ScriptBuilder::new();
        builder.push_data(signature.to_bytes());
        Self::new(builder.build())
    }

    pub fn from_message_and_keypair(message: Bytes, keypair: &mut KeyPair) -> Self {
        let signature = keypair.sign(&message);
        Self::from_signature(signature)
    }

    // other methods
}