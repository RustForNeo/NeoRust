use crate::{
	contract::{contract_error::ContractError, iterator::Iterator},
	protocol::{
		core::{
			responses::{
				invocation_result::InvocationResult, neo_response_aliases::NeoInvokeFunction,
			},
			stack_item::StackItem,
		},
		neo_rust::NeoRust,
	},
	script::{op_code::OpCode, script_builder::ScriptBuilder},
	transaction::{signer::Signer, transaction_builder::TransactionBuilder},
	types::{call_flags::CallFlags, contract_parameter::ContractParameter, H160Externsion},
	utils::bytes::BytesExtern,
};
use byte_slice_cast::AsByteSlice;
use primitive_types::{H160, H256};
use serde::{Deserialize, Serialize};
use std::{error::Error, iter::Iterator};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmartContract {
	script_hash: H160,
}

impl SmartContract {
	const DEFAULT_ITERATOR_COUNT: usize = 100;

	pub fn new(script_hash: H160) -> Self {
		Self { script_hash }
	}

	pub fn invoke_function(
		&self,
		function: &str,
		params: Vec<Option<ContractParameter>>,
	) -> Result<TransactionBuilder, ContractError> {
		let script = self.build_invoke_function_script(function, params)?;
		Ok(TransactionBuilder::new().script(script))
	}

	pub fn build_invoke_function_script(
		&self,
		function: &str,
		params: Vec<Option<ContractParameter>>,
	) -> Result<Script, ContractError> {
		if function.is_empty() {
			return Err(ContractError::InvalidNeoName("Function name cannot be empty".to_string()))
		}

		let script = ScriptBuilder::new()
			.contract_call(self.script_hash.clone(), function, params.as_slice(), CallFlags::None)
			.build();

		Ok(script)
	}

	pub async fn call_function_returning_string(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
	) -> Result<String, ContractError> {
		let output = self.call_invoke_function(function, params, vec![]).await?.get_result();
		self.throw_if_fault_state(&output)?;

		let item = output.stack[0].clone();
		item.as_str()
			.ok_or_else(|| ContractError::UnexpectedReturnType("String".to_string(), &item))
	}

	pub async fn call_function_returning_int(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
	) -> Result<i32, ContractError> {
		let output = self.call_invoke_function(function, params, vec![]).await?.get_result();
		self.throw_if_fault_state(&output)?;

		let item = output.stack[0].clone();
		item.as_i32()
			.ok_or_else(|| ContractError::UnexpectedReturnType("Int".to_string(), &item))
	}

	pub async fn call_function_returning_bool(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
	) -> Result<bool, ContractError> {
		let output = self.call_invoke_function(function, params).await?.get_result();
		self.throw_if_fault_state(&output)?;

		let item = output.stack[0].clone();
		item.as_bool()
			.ok_or_else(|| ContractError::UnexpectedReturnType("Bool".to_string(), &item))
	}

	// Other methods

	pub async fn call_invoke_function(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
		signers: Vec<dyn Signer>,
	) -> Result<NeoInvokeFunction, dyn Error> {
		if function.is_empty() {
			return Err(ContractError::InvalidNeoName("Function cannot be empty".to_string()))
		}

		NeoRust::instance()
			.invoke_function(&self.script_hash, function.into(), params, signers)
			.await
	}

	pub fn throw_if_fault_state(&self, output: &InvocationResult) -> Result<(), ContractError> {
		if output.has_state_fault() {
			Err(ContractError::UnexpectedReturnType(output.exception.unwrap(), None))
		} else {
			Ok(())
		}
	}

	// Other methods like `call_function_returning_xxx`, iterators, etc.
	pub async fn call_function_returning_script_hash(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
	) -> Result<H160, ContractError> {
		let output = self.call_invoke_function(function, params, vec![]).await?.get_result();
		self.throw_if_fault_state(&output)?;

		let item = &output.stack[0];
		item.as_bytes()
			.and_then(H160::from_slice)
			.ok_or_else(|| ContractError::UnexpectedReturnType("Script hash".to_string(), item))
	}

	pub async fn call_function_returning_iterator<T>(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
		mapper: impl Fn(StackItem) -> Result<T, ContractError>,
	) -> Result<NeoIterator<T>, ContractError> {
		let output = self.call_invoke_function(function, params, vec![]).await?.get_result();
		self.throw_if_fault_state(&output)?;

		let item = &output.stack[0];
		let interface = item
			.as_interop()
			.ok_or_else(|| ContractError::UnexpectedReturnType("Iterator".to_string(), item))?;

		let session_id = output
			.session_id
			.ok_or(ContractError::InvalidNeoNameServiceRoot("No session ID".to_string()))?;

		Ok(Iterator::new(session_id, interface.iterator_id, mapper))
	}

	pub async fn call_function_and_unwrap_iterator<T>(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
		max_items: usize,
		mapper: impl Fn(StackItem) -> T,
	) -> Result<Vec<T>, ContractError> {
		let script = ScriptBuilder::new().contract_call(
			self.script_hash.clone(),
			function,
			params.iter().filter_map(|p| Some(p)).collect(),
			max_items,
		)?;

		let output = NeoRust::instance()
			.invoke_script(script.to_bytes().to_hex(), vec![])
			.await?
			.get_result();

		self.throw_if_fault_state(&output)?;

		let items = output.stack[0].to_array()?.into_iter().map(mapper).collect();

		Ok(items)
	}

	pub fn calc_native_contract_hash(contract_name: &str) -> Result<H160, dyn Error> {
		Self::calc_contract_hash(H160::zero(), 0, contract_name)
	}

	pub fn calc_contract_hash(
		sender: H160,
		nef_checksum: u32,
		contract_name: &str,
	) -> Result<H160, dyn Error> {
		let mut script = ScriptBuilder::new()
			.op_code(&[OpCode::Abort])
			.push_data(sender.to_vec())
			.unwrap()
			.push_integer(nef_checksum as i64)
			.unwrap()
			.push_data(contract_name.as_bytes().to_vec());

		Ok(H160::from_slice(script.to_bytes().as_slice()))
	}
}
