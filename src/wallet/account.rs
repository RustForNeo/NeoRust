use std::error::Error;
use crypto::hmac::Hmac;
use crypto::pbkdf2::pbkdf2;
use num_bigint::BigInt;
use primitive_types::H160;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::Sha256;
use crate::crypto::key_pair::KeyPair;
use crate::script::verification_script::VerificationScript;
use crate::types::{Address};
use crate::wallet::nep6account::NEP6Account;
use crate::wallet::nep6contract::NEP6Contract;
use crate::wallet::wallet_error::WalletError;

#[derive(Debug, PartialEq, Eq, Clone, Copy,Serialize, Deserialize)]
pub struct Account {
    pub key_pair: Option<KeyPair>,
    pub address: Address,
    pub label: Option<String>,
    pub verification_script: Option<VerificationScript>,
    pub is_locked: bool,
    pub encrypted_private_key: Option<String>,
    pub signing_threshold: Option<u32>,
    pub nr_participants: Option<u32>,
}

impl Account {

    pub fn new(address: Address, label: Option<String>) -> Self {
        Self {
            key_pair: None,
            address,
            label,
            verification_script: None,
            is_locked: false,
            encrypted_private_key: None,
            signing_threshold: None,
            nr_participants: None,
        }
    }

    pub fn from_key_pair(key_pair: KeyPair) -> Result<Self, dyn Error> {
        let address = key_pair.to_address()?;

        Ok(Self {
            key_pair: Some(key_pair),
            address,
            label: Some(address.to_string()),
            verification_script: Some(VerificationScript::from_pubkey(key_pair.public_key)),
            is_locked: false,
            encrypted_private_key: None,
            signing_threshold: None,
            nr_participants: None,
        })
    }

    pub fn from_nep6(nep6: NEP6Account) -> Result<Self, WalletError> {

        let verification_script = nep6.contract.as_ref().and_then(|c| {
            VerificationScript::from_hex(&c.script.unwrap())
        });

        let signing_threshold = nep6.contract.as_ref().and_then(|c| {
            c.parameters.iter().filter(|p| p.param_type == ParamType::Signature)
                .count() as u32
        });

        Ok(Self {
            key_pair: None,
            address: nep6.address,
            label: nep6.label,
            verification_script,
            is_locked: false,
            encrypted_private_key: nep6.key,
            signing_threshold,
            nr_participants: None,
        })

    }

    pub fn from_multisig(address: Address, threshold: u32, num_keys: u32) -> Self {
        Self {
            key_pair: None,
            address,
            label: None,
            verification_script: None,
            is_locked: false,
            encrypted_private_key: None,
            signing_threshold: Some(threshold),
            nr_participants: Some(num_keys),
            // ..
        }
    }

    pub fn from_script_hash(script_hash: H160) -> Result<Self, WalletError> {
        let address = script_hash.to_address()?;
        Ok(Self::new(address, None))
    }

    // Other methods
    pub fn set_label(&mut self, label: String) {
        self.label = Some(label);
    }

    pub fn get_address(&self) -> Result<Address, WalletError> {
        match &self.key_pair {
            Some(key_pair) => Ok(key_pair.public_key.to_address()?),
            None => Err(WalletError::NoKeyPair),
        }
    }

    pub fn decrypt_private_key(&mut self, password: &str) -> Result<(), Error> {
        let key_pair = decrypt(password, self.encrypted_private_key.as_ref().unwrap())?;
        self.key_pair = Some(key_pair);
        self.encrypted_private_key = None;
        Ok(())
    }
    fn decrypt(password: &str, encrypted_hex: &str) -> Result<KeyPair, Error> {

        // Parse encrypted key
        let encrypted = Nep2Key::from_hex(encrypted_hex)?;

        // Derive decryption key
        let mut hmac = Hmac::new(Sha256::new(), password.as_bytes());
        let mut derived_key = vec![0u8; 64];
        pbkdf2(&mut hmac, &mut derived_key, encrypted.n, encrypted.r, encrypted.p);

        // Decrypt private key
        let mut decrypted_priv_key = vec![0u8; 32];
        aes_decrypt_ecb(derived_key.as_slice(), encrypted.ciphertext, &mut decrypted_priv_key);

        // Recover key pair
        let private_key = BigInt::from_bytes(decrypted_priv_key.as_slice());
        let public_key = ECPoint::from_private_key(&private_key)?;

        Ok(KeyPair {
            private_key,
            public_key,
        })
    }
    pub fn decrypt_private_key(&mut self, password: &str) -> Result<(), Error> {

        // Validate encrypted key exists
        if self.encrypted_private_key.is_none() {
            return Err(Error::NoEncryptedKey);
        }

        // Call decrypt function
        let encrypted_hex = self.encrypted_private_key.as_ref().unwrap();
        let key_pair = decrypt(password, encrypted_hex)?;

        // Update fields
        self.key_pair = Some(key_pair);
        self.encrypted_private_key = None;

        Ok(())
    }

    pub fn is_default(&self) -> bool {
        self.wallet.as_ref()
            .and_then(|w| w.is_default_account(&self.address))
            .unwrap_or(false)
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    pub fn to_nep6(&self) -> Result<NEP6Account, Error> {
        let contract = if let Some(script) = &self.verification_script {
            let params = // get params from script
                Some(NEP6Contract {
                    script: Some(hex::encode(script)),
                    is_deployed: false,
                    nep6_parameters: params,
                });
        } else {
            None
        };

        Ok(NEP6Account {
            address: self.address.to_string(),
            label: self.label.clone(),
            is_default: false,
            lock: false,
            key: self.encrypted_private_key.clone(),
            contract: None,
            extra: None,
        })
    }
}

impl Serialize for Account{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {

    }
}


impl Deserialize for Account {
    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {

    }
}

//
//
// impl Serializable for Account {
//
//     fn serialize(&self) -> Result<Vec<u8>, Error> {
//         let mut buf = Vec::new();
//         buf.write_bool(self.is_locked);
//         buf.write_bytes(&self.address.to_bytes()?);
//         // Serialize other fields
//
//         Ok(buf)
//     }

    // fn deserialize(bytes: &[u8]) -> Result<Self, Error> {
    //     let mut buf = Cursor::new(bytes);
    //     let is_locked = buf.read_bool()?;
    //     let address = Address::from_bytes(buf.read_bytes(20)?);
    //     // Deserialize other fields
    //     Ok(Self::new(address, None))
    // }

}