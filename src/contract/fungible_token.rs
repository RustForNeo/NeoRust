// fungible_token

use primitive_types::H160;
use crate::contract::nns_name::NNSName;
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::types::Bytes;
use crate::wallet::account::Account;
use crate::wallet::wallet::Wallet;

pub trait FungibleToken {

    const BALANCE_OF: &'static str = "balanceOf";
    const TRANSFER: &'static str = "transfer";

     async fn get_balance(&self, account: &Account) -> u64 {
        self.get_balance(&account.script_hash()).await
    }

     async fn get_balance_hash256(&self, script_hash: &H160) -> u64 {
        self.call_function_u64(FungibleToken::BALANCE_OF, vec![script_hash]).await
    }

     async fn transfer(&self, from: &Account, to: &H160, amount: u64) -> TransactionBuilder {
        let script = self.build_transfer_script(from.script_hash(), to, amount).await;
        TransactionBuilder::new()
            .script(script)
            .signer(from)
    }

     async fn build_transfer_script(&self, from: &H160, to: &H160, amount: u64) -> Bytes {
        self.build_invoke_function_script(
            FungibleToken::TRANSFER,
            vec![from, to, amount]
        )
            .await
    }

    // other methods
     async fn get_wallet_balance(&self, wallet: &Wallet) -> u64 {
        let mut total = 0;

        for account in wallet.accounts() {
            let balance = self.get_balance(&account).await;
            total += balance;
        }

        total
    }
     async fn transfer_to_nns(&self, from: &Account, to: &NNSName, amount: u64) -> TransactionBuilder {

        let resolver = NnsResolver::default();
        let to_script_hash = resolver.resolve(to).await;

        self.transfer(from, &to_script_hash, amount).await
    }

    async fn transfer_to_nns_hash160(&self, from: &Account, to: &H160, amount: u64) -> TransactionBuilder {
        self.transfer(from, to, amount).await
    }

     async fn mint(&self, to: &Account, amount: u64) -> TransactionBuilder {

        let script = self.build_mint_script(to, amount).await;

        TransactionBuilder::new()
            .script(script)
    }
     async fn build_mint_script(&self, to: &Account, amount: u64) -> Bytes {
        self.build_invoke_function_script(
            FungibleToken::MINT,
            vec![to.script_hash(), amount]
        )
            .await
    }
}