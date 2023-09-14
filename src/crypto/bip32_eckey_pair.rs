use bip32::DerivationPath;
use bitcoin::bip32::ExtendedPrivKey;
use bitcoin::Network;
use secp256k1::Secp256k1;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Debug, Clone, PartialEq, Eq,Serialize, Deserialize)]
pub struct Bip32ECKeyPair {
    extended_priv_key: ExtendedPrivKey
}

impl Bip32ECKeyPair {

    const HARDENED_BIT: u32 = 0x80000000;

    pub fn from_seed(seed: &[u8]) -> Self {
        let master = ExtendedPrivKey::new_master(Network::Bitcoin, seed);
        Self { extended_priv_key: master.unwrap() }
    }

    pub fn derive(&self, path: &DerivationPath) -> Self {
        let child = self.extended_priv_key.derive_priv(&Secp256k1::new(),path).expect("Invalid path");
        Self { extended_priv_key: child }
    }

    pub fn is_hardened(index: u32) -> bool {
        index & Self::HARDENED_BIT != 0
    }

}