use p256::pkcs8::der::Decode;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use crate::contract::contract_error::ContractError;
use crate::protocol::core::stack_item::StackItem;
use crate::protocol::neo_rust::NeoRust;
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::types::contract_parameter::ContractParameter;


#[repr(u8)]
enum RecordType {
    None = 0,
    Txt = 1,
    A = 2,
    Aaaa = 3,
    Cname = 4,
    Srv = 5,
    Url = 6,
    Oauth = 7,
    Ipfs = 8,
    Email = 9,
    Dnssec = 10,
    Tlsa = 11,
    Smimea = 12,
    Hippo = 13,
    Http = 14,
    Sshfp = 15,
    Onion = 16,
    Xmpp = 17,
    Magnet = 18,
    Tor = 19,
    I2p = 20,
    Git = 21,
    Keybase = 22,
    Briar = 23,
    Zcash = 24,
    Mini = 25,
}

// NameState struct

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NameState {
    pub name: String,
    pub expiration: u32,
    pub admin: Option<H160>,
}

pub struct NeoNameService {
    script_hash: H160,
    client: NeoRust,
}

impl NeoNameService {

    const ADD_ROOT: &'static str = "addRoot";
    const ROOTS: &'static str = "roots";
    const SET_PRICE: &'static str = "setPrice";
    const GET_PRICE: &'static str = "getPrice";
    const IS_AVAILABLE: &'static str = "isAvailable";
    const REGISTER: &'static str = "register";
    const RENEW: &'static str = "renew";
    const SET_ADMIN: &'static str = "setAdmin";
    const SET_RECORD: &'static str = "setRecord";
    const GET_RECORD: &'static str = "getRecord";
    const GET_ALL_RECORDS: &'static str = "getAllRecords";
    const DELETE_RECORD: &'static str = "deleteRecord";
    const RESOLVE: &'static str = "resolve";
    const PROPERTIES: &'static str = "properties";

    const NAME_PROPERTY: &'static [u8] = b"name";
    const EXPIRATION_PROPERTY: &'static [u8] = b"expiration";
    const ADMIN_PROPERTY: &'static [u8] = b"admin";

    pub fn new(script_hash: H160, client: Box<NeoRust>) -> Self {
        Self {
            script_hash,
            client: *client,
        }
    }

    // Implementation

    async fn add_root(&self, root: &str) -> Result<TransactionBuilder, ContractError> {
        let args = vec![root.to_string().into()];
        self.invoke_function(Self::ADD_ROOT, args)
    }

    async fn get_roots(&self) -> Result<Vec<String>, ContractError> {
        let args = vec![];
        let roots = self.call_function_iter(Self::ROOTS, args, |item| {
            item.get_str()
        })?;
        Ok(roots)
    }

    async fn get_symbol(&self) -> Result<String, ContractError> {
        Ok("NNS".to_string())
    }

    async fn get_decimals(&self) -> Result<u8, ContractError> {
        Ok(0)
    }

    // Register a name

    pub async fn register(&self, name: &str, owner: H160) -> Result<TransactionBuilder, ContractError> {
        self.check_domain_name_availability(name, true).await?;

        let args = vec![name.into(), owner.into()];
        self.invoke_function(Self::REGISTER, args)
    }

// Set admin for a name

    pub async fn set_admin(&self, name: &str, admin: H160) -> Result<TransactionBuilder, ContractError> {
        self.check_domain_name_availability(name, true).await?;

        let args = vec![name.into(), admin.into()];
        self.invoke_function(Self::SET_ADMIN, args)
    }

// Set record

    pub async fn set_record(&self, name: &str, record_type: RecordType, data: &str) -> Result<TransactionBuilder, ContractError> {
        let args = vec![
            name.into(),
            record_type as u8.into(),
            data.into()
        ];

        self.invoke_function(Self::SET_RECORD, args)
    }

// Delete record

    pub async fn delete_record(&self, name: &str, record_type: RecordType) -> Result<TransactionBuilder, ContractError> {
        let args = vec![name.into(), record_type as u8.into()];
        self.invoke_function(Self::DELETE_RECORD, args)
    }

    async fn owner_of(&self, name: &[u8]) -> Result<H160, ContractError> {
        self.call_function("ownerOf", vec![name.into()])
            .await?.as_address()
            .map(Into::into)
    }

    pub async fn get_price(&self, length: u32) -> Result<u32, ContractError> {
        let args = vec![length.into()];
        self.call_function::<i64>(Self::GET_PRICE, args)
            .await?
            .try_into()
            .map_err(Into::into)
    }

    pub async fn is_available(&self, name: &str) -> Result<bool, ContractError> {
        let args = vec![name.into()];
        self.call_function::<bool>(Self::IS_AVAILABLE, args)
            .await
    }
    pub async fn renew(&self, name: &str, years: u32) -> Result<TransactionBuilder, ContractError> {
        self.check_domain_name_availability(name, true).await?;

        let args = vec![name.into(), years.into()];
        self.invoke_function(Self::RENEW, args)
    }


    // Other methods...
    async fn get_name_state(&self, name: &[u8]) -> Result<NameState, ContractError> {
        let args = vec![name.into()];
        let result = self.invoke_function(Self::PROPERTIES, args).await?;

        let map = result.as_map()?;
        let name = map.get(Self::NAME_PROPERTY)?.as_str()?;
        let expiration = map.get(Self::EXPIRATION_PROPERTY)?.as_i64()? as u32;
        let admin = map.get(Self::ADMIN_PROPERTY)?.as_address()?;

        Ok(NameState {
            name,
            expiration,
            admin: admin.map(Into::into),
        })
    }
    async fn check_domain_name_availability(&self, name: &str, should_be_available: bool) -> Result<(), ContractError> {
        let is_available = self.is_available(name).await?;

        if &should_be_available && !&is_available {
            return Err("Domain name already taken".into());
        } else if !should_be_available && is_available {
            return Err("Domain name not registered".into());
        }

        Ok(())
    }
    async fn invoke_function(&self, operation: &str, args: Vec<StackItem>) -> Result<TransactionBuilder, ContractError> {
        let script_hash = &self.script_hash;
        let tx_builder = TransactionBuilder::call_contract(script_hash, operation, args);
        Ok(tx_builder)
    }

    async fn call_function<T: Decode>(&self, operation: &str, args: Vec<ContractParameter>) -> Result<T, ContractError> {
        let script_hash = &self.script_hash;

        let result = self.client
            .invoke_function(script_hash, operation.to_string(), args, vec![])
            .await?
            .as_interop()?;

        result.decode()
    }

}