use crate::{
	contract::{contract_error::ContractError, traits::smartcontract::SmartContractTrait},
	protocol::{
		core::{neo_trait::NeoTrait, stack_item::StackItem},
		http_service::HttpService,
		neo_rust::NeoRust,
	},
	transaction::transaction_builder::TransactionBuilder,
	types::{PublicKey, ValueExtension},
	utils::*,
};
use async_trait::async_trait;
use num_enum::TryFromPrimitive;
use p256::{elliptic_curve::sec1::ToEncodedPoint, pkcs8::der::Encode};
use primitive_types::H160;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleManagement {
	#[serde(deserialize_with = "deserialize_address")]
	#[serde(serialize_with = "serialize_address")]
	script_hash: H160,
}

impl RoleManagement {
	const NAME: &'static str = "RoleManagement";
	const SCRIPT_HASH: H160 = Self::calc_native_contract_hash(Self::NAME).unwrap(); // compute hash

	pub fn new() -> Self {
		Self { script_hash: Self::SCRIPT_HASH }
	}

	pub async fn get_designated_by_role(
		&self,
		role: Role,
		block_index: i32,
	) -> Result<Vec<PublicKey>, ContractError> {
		self.check_block_index_validity(block_index).await.unwrap();

		let invocation = self
			.call_invoke_function(
				"getDesignatedByRole",
				vec![role.into(), block_index.into()],
				vec![],
			)
			.await
			.unwrap();

		let designated = invocation.stack[0]
			.as_array()
			.unwrap()
			.into_iter()
			.map(|item| PublicKey::try_from(item.as_bytes().unwrap().as_slice()).unwrap())
			.collect();

		Ok(designated)
	}

	async fn check_block_index_validity(&self, block_index: i32) -> Result<(), ContractError> {
		if block_index < 0 {
			return Err(ContractError::InvalidNeoName("Block index must be positive".to_string()))
		}

		let current_block_count = NeoRust::instance().get_block_count().request().await.unwrap();

		if block_index > current_block_count as i32 {
			return Err(ContractError::InvalidNeoName(format!(
				"Block index {} exceeds current block count {}",
				block_index, current_block_count
			)))
		}

		Ok(())
	}

	pub fn designate_as_role(
		&self,
		role: Role,
		pub_keys: Vec<PublicKey>,
	) -> Result<TransactionBuilder, ContractError> {
		if pub_keys.is_empty() {
			return Err(ContractError::InvalidNeoName(
				"At least 1 public key is required".to_string(),
			))
		}

		let params: Vec<_> = pub_keys.into_iter().map(|key| key.to_value()).collect();

		self.invoke_function("designateAsRole", vec![role.into(), params.into()])
	}
}

#[async_trait]
impl SmartContractTrait for RoleManagement {
	fn script_hash(&self) -> H160 {
		self.script_hash.clone()
	}

	fn set_script_hash(&mut self, script_hash: H160) {
		self.script_hash = script_hash;
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Role {
	Oracle,
	Policy,
	Validator,
	StateRootValidator,
	PriceFeedOracle,
	FeeCollector,
	ComplianceOfficer,
}

impl Role {
	pub const fn byte(self) -> u8 {
		self as u8
	}
}

impl From<Role> for StackItem {
	fn from(role: Role) -> Self {
		StackItem::Integer { value: role.byte() as i64 }
	}
}
