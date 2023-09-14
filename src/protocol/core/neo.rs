use bitcoin::PrivateKey;
use p256::PublicKey;
use primitive_types::{H160, H256};
use crate::transaction::signer::Signer;
use crate::types::contract_parameter::ContractParameter;
use crate::wallet::account::Account;

pub trait Neo {

    // Blockchain methods

    fn get_best_block_hash(&self) -> Result<H256, Error>;

    fn get_block(&self, hash: H256) -> Result<Block, Error>;

    fn get_block_count(&self) -> Result<u32, Error>;

    fn get_block_hash(&self, index: u32) -> Result<H256, Error>;

    fn get_raw_block(&self, hash: H256) -> Result<Vec<u8>, Error>;

    fn get_transaction(&self, hash: H256) -> Result<Transaction, Error>;

    fn get_contract_state(&self, hash: H160) -> Result<ContractState, Error>;


    // Node methods

    fn get_peers(&self) -> Result<Vec<Peer>, Error>;

    fn send_raw_transaction(&self, tx: &[u8]) -> Result<H256, Error>;

    fn get_connection_count(&self) -> Result<usize, Error>;

    fn get_version(&self) -> Result<VersionInfo, Error>;

    fn submit_block(&self, block: &[u8]) -> Result<(), Error>;


    // Smart contract methods

    fn invoke_function(
        &self,
        contract: H160,
        method: &str,
        params: &[ContractParameter],
        signers: &[Signer]
    ) -> Result<InvokeResult, Error>;

    fn invoke_script(
        &self,
        script: &[u8],
        signers: &[Signer],
    ) -> Result<InvokeResult, Error>;

    // Wallet methods

    fn get_account(&self, script_hash: &H160) -> Result<Account, Error>;

    fn send_tokens(
        &self,
        from: &Account,
        to: &Account,
        amount: u64
    ) -> Result<H256, Error>;

    fn open_wallet(&self, path: &str, password: &str) -> Result<(), Error>;

    fn close_wallet(&self) -> Result<(), Error>;

    fn get_public_key(&self, script_hash: &H160) -> Result<PublicKey, Error>;

    fn get_private_key(&self, script_hash: &H160) -> Result<PrivateKey, Error>;

}