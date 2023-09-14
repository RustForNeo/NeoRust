use bip32::{Mnemonic, Seed};
use secp256k1::KeyPair;
use crate::wallet::account::Account;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bip39Account {
    mnemonic: String,
    account: Account,
}

impl Bip39Account {

    pub fn new(mnemonic: String, account: Account) -> Self {
        Self { mnemonic, account }
    }

    pub fn create(password: &str) -> Result<Self, bip39::Error> {
        let mnemonic = Mnemonic::new(Default::default(), Default::default())?;
        let seed = Seed::new(&mnemonic, password)?;

        let private_key = seed.as_bytes();
        let key_pair = KeyPair::from_private_key(private_key)?;

        let account = Account::from_key_pair(key_pair)?;

        Ok(Self::new(mnemonic.phrase().into(), account))
    }

    pub fn from_phrase(password: &str, phrase: &str) -> Result<Self, bip39::Error> {

        // Parse phrase into mnemonic
        let mnemonic = Mnemonic::from_phrase(phrase)?;

        // Generate seed from mnemonic and password
        let seed = Seed::new(&mnemonic, password)?;

        // Derive private key from seed
        let private_key = seed.as_bytes();

        // Generate key pair from private key
        let key_pair = KeyPair::from_private_key(private_key)?;

        // Create account from key pair
        let account = Account::from_key_pair(key_pair)?;

        // Construct Bip39Account
        let bip39_account = Self {
            mnemonic: mnemonic.phrase(),
            account,
        };

        Ok(bip39_account)

    }

}