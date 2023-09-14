use secp256k1::ecdsa::Signature;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::crypto::key_pair::ECKeyPair;
use crate::script::invocation_script::InvocationScript;
use crate::script::verification_script::VerificationScript;
use crate::serialization::binary_reader::BinaryReader;
use crate::serialization::binary_writer::BinaryWriter;
use crate::transaction::transaction_error::TransactionError;
use crate::types::Bytes;
use crate::types::contract_parameter::ContractParameter;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Witness {
    pub invocation_script: InvocationScript,
    pub verification_script: VerificationScript,
}

impl Witness {

    pub fn new(invocation_script: InvocationScript, verification_script: VerificationScript) -> Self {
        Self {
            invocation_script,
            verification_script
        }
    }

    pub fn create(message: Bytes, keypair: ECKeyPair) -> Result<Self, TransactionError> {
        let invocation_script = InvocationScript::from_message_and_keypair(message, keypair)?;
        let verification_script = VerificationScript::from_pubkey(keypair.public_key);
        Ok(Self { invocation_script, verification_script })
    }

    pub fn create_multisig(signatures: Vec<Signature>, verification_script: VerificationScript) -> Result<Self, TransactionError> {
        let invocation_script = InvocationScript::from_signatures(signatures)?;
        Ok(Self { invocation_script, verification_script})
    }

    // Other constructors
    pub fn empty() -> Self {
        Self {
            invocation_script: InvocationScript::default(),
            verification_script: VerificationScript::default()
        }
    }

    pub fn from_scripts(invocation_script: InvocationScript, verification_script: VerificationScript) -> Self {
        Self {
            invocation_script,
            verification_script
        }
    }

    pub fn contract_witness(params: &[ContractParameter]) -> Self {
        let invocation_script = InvocationScript::from_params(params);
        Self {
            invocation_script,
            verification_script: VerificationScript::default()
        }
    }
}

impl Serialize for Witness {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut writer = BinaryWriter::new();
        // self.serialize(&mut writer);
        self.invocation_script.serialize(&writer);
        self.verification_script.serialize(&writer);
        let bytes = writer.to_bytes();
        serializer.serialize_bytes(&bytes)
    }
}

impl Deserialize for Witness{
    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let bytes = <&[u8]>::deserialize(deserializer)?;
        let mut reader = BinaryReader::from_bytes(bytes);
        // Self::deserialize(&mut reader)
        let invocation_script = InvocationScript::deserialize(&mut reader).map_err(|e| serde::de::Error::custom(e.to_string()))?;
        let verification_script = VerificationScript::deserialize(&mut reader).map_err(|e| serde::de::Error::custom(e.to_string()))?;
        Ok(Self {
            invocation_script,
            verification_script
        })
    }
}