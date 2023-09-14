use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Mutex;
use primitive_types::{H160, H256};
use serde::{Deserialize, Serialize};
use crate::contract::contract_error::ContractError;
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::types::contract_parameter::ContractParameter;
use crate::wallet::account::Account;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonFungibleToken {
    script_hash: H160,
    name: Option<String>,
    symbol: Option<String>,
    decimals: u8,

    owner_of_cache: Mutex<HashMap<H256, H160>>,
    tokens_of_cache: Mutex<HashMap<H160, HashSet<H256>>>,

    on_transfer_handler: Option<Rc<dyn FnMut(H160, H160, H256, ContractParameter) + 'static>>,

    admin: H160,
    max_supply: Option<u32>,
}
impl NonFungibleToken {

    // Constants
    const OWNER_OF: &'static str = "ownerOf";
    const TRANSFER: &'static str = "transfer";

    // Methods

    async fn mint(&mut self, to: &H160, token_id: &H256) -> Result<TransactionBuilder, ContractError> {

        // Enforce max supply if set
        if let Some(max_supply) = &self.max_supply {
            let total_supply = self.total_supply().await?;
            if &total_supply >= max_supply {
                return Err(ContractError::RuntimeError("Max supply reached".to_string()));
            }
        }

        // Build transaction
        let tx = self.invoke_function("mint", vec![to.into(), token_id.into()])?;

        // Update caches
        self.owner_of_cache.lock().insert(*token_id, *to);
        self.tokens_of_cache.lock().entry(*to).or_default().insert(*token_id);

        Ok(tx)
    }

    async fn name(&mut self) -> Result<String, ContractError> {
        if let Some(name) = &self.name {
            return Ok(name.clone());
        }

        let name = self.call_contract_method("name").await?;
        self.name = Some(name.clone());
        Ok(name)
    }

    async fn symbol(&mut self) -> Result<String, ContractError> {
        if let Some(symbol) = &self.symbol {
            return Ok(symbol.clone());
        }

        let symbol = self.call_contract_method("symbol").await?;
        self.symbol = Some(symbol.clone());
        Ok(symbol)
    }

    async fn decimals(&self) -> Result<u8, ContractError> {
        Ok(self.clone().decimals)
    }

    async fn total_supply(&self) -> Result<u32, ContractError> {
        self.call_contract_method("totalSupply").await
    }

    async fn owner_of(&self, token_id: &H256) -> Result<H160, ContractError> {
        if let Some(owner) = self.owner_of_cache.lock().get(token_id) {
            return Ok(owner.clone());
        }

        let owner = self.call_contract_method("ownerOf", vec![token_id.into()]).await?;
        self.owner_of_cache.lock().insert(*token_id, owner.clone());
        Ok(owner)
    }

    // pub async fn owner_of(&self, token_id: Vec<u8>) -> Result<H160, ContractError> {
    //     self.throw_if_divisible().await?;
    //
    //     let owner_addr = self
    //         .call_function_returning_address(Self::OWNER_OF, vec![token_id.into()])
    //         .await?;
    //
    //     H160::from_address(&owner_addr)
    // }

    async fn balance_of(&self, owner: &H160) -> Result<u32, ContractError> {
        let tokens = self.tokens_of(owner).await?;
        Ok(tokens.len() as u32)
    }

    async fn tokens_of(&self, owner: &H160) -> Result<Vec<H256>, ContractError> {
        if let Some(tokens) = self.tokens_of_cache.lock().get(owner) {
            return Ok(tokens.iter().cloned().collect());
        }

        let tokens = self.call_contract_method("tokensOf", vec![owner.into()])
            .await?
            .to_array()?
            .iter()
            .map(|token| token.as_bytes().to_vec().into())
            .collect();

        self.tokens_of_cache.lock().insert(*owner, tokens.clone());

        Ok(tokens)
    }


    pub async fn transfer(&self, from: &Account, to: &H160, token_id: Vec<u8>, data: Option<ContractParameter>) -> Result<TransactionBuilder, ContractError> {
        self.transfer(to, token_id, data).await?.signer(from.to_signer())
    }

    pub async fn transfer(&self, to: &H160, token_id: Vec<u8>, data: Option<ContractParameter>) -> Result<TransactionBuilder, ContractError> {
        self.throw_if_divisible().await?;

        self.invoke_function(Self::TRANSFER, vec![
            to.to_stack_item(),
            token_id.into(),
            data.unwrap_or_default().into()
        ])
    }

    async fn transfer(&mut self, from: &H160, to: &H160, token_id: &H256) -> Result<TransactionBuilder, ContractError> {

        // Verify from owns token
        assert_eq!(self.owner_of(token_id).await?, *from, "Not owner");

        // Build transaction
        let tx = self.invoke_function("transfer", vec![from.into(), to.into(), token_id.into()])?;

        // Update caches
        self.owner_of_cache.lock().insert(*token_id, *to);
        self.tokens_of_cache.lock().entry(*from).and_modify(|tokens| {
            tokens.remove(token_id);
        });
        self.tokens_of_cache.lock().entry(*to).or_default().insert(*token_id);

        // Call handler
        if let Some(handler) = &self.on_transfer_handler {
            handler(from.clone(), to.clone(), token_id.clone(), None);
        }

        Ok(tx)
    }
    async fn throw_if_divisible(&self) -> Result<(), ContractError> {
        if self.get_decimals().await? != 0 {
            Err(ContractError::InvalidArgError("Divisible NFT".to_string()))
        } else {
            Ok(())
        }
    }

    // Other methods

    pub async fn tokens_of(&self, owner: &H160) -> Result<Iterator<Vec<u8>>, ContractError> {
        self.call_function_returning_iterator(Self::TOKENS_OF, vec![owner.to_stack_item()], |item| {
            item.as_bytes()
        }).await
    }

    pub async fn properties(&self, token_id: Vec<u8>) -> Result<HashMap<String, String>, ContractError> {

        let result = self.call_invoke_function(Self::PROPERTIES, vec![token_id.into()])
            .await?
            .get_result();

        let map = result.to_map()?;
        let props = map.iter()
            .filter_map(|(k, v)| v.as_str().map(|v| (k.clone(), v.to_owned())))
            .collect();

        Ok(props)
    }
}

impl Token for NonFungibleToken {

    fn new(script_hash: H160) -> Self {
        Self {
            script_hash,
            name: None,
            symbol: None,
            decimals: 0,

            owner_of_cache: Default::default(),
            tokens_of_cache: Default::default(),

            on_transfer_handler: None,

            admin: script_hash, // Admin is deployer by default
            max_supply: None,
        }
    }

    async fn balance_of(&self, owner: &H160) -> Result<i32, ContractError> {
        self.call_function_returning_int(Self::BALANCE_OF, vec![owner.to_stack_item()])
            .await
    }
}