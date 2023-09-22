use crate::{
	contract::{contract_error::ContractError, iterator::NeoIterator},
	neo_error::NeoError,
	protocol::{
		core::{
			neo_trait::NeoTrait,
			responses::{contract_manifest::ContractManifest, invocation_result::InvocationResult},
			stack_item::StackItem,
		},
		http_service::HttpService,
		neo_rust::NeoRust,
	},
	script::{op_code::OpCode, script_builder::ScriptBuilder},
	transaction::{signer::Signer, transaction_builder::TransactionBuilder},
	types::{call_flags::CallFlags, contract_parameter::ContractParameter, Bytes, H160Externsion},
};
use async_trait::async_trait;
use primitive_types::H160;

#[async_trait]
pub trait SmartContractTrait: Send + Sync {
	const DEFAULT_ITERATOR_COUNT: usize = 100;

	async fn name(&self) -> String {
		self.get_manifest().await.name.unwrap()
	}
	fn set_name(&mut self, name: String) {
		panic!("Cannot set name for NNS")
	}

	fn script_hash(&self) -> H160;

	fn set_script_hash(&mut self, script_hash: H160) {
		panic!("Cannot set script hash for NNS")
	}

	async fn invoke_function(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
	) -> Result<TransactionBuilder, ContractError> {
		let script = self.build_invoke_function_script(function, params).await.unwrap();
		let mut builder = TransactionBuilder::new();
		builder.set_script(script);
		Ok(builder)
	}

	async fn build_invoke_function_script(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
	) -> Result<Bytes, ContractError> {
		if function.is_empty() {
			return Err(ContractError::InvalidNeoName("Function name cannot be empty".to_string()))
		}

		let script = ScriptBuilder::new()
			.contract_call(&self.script_hash(), function, params.as_slice(), CallFlags::None)
			.unwrap()
			.to_bytes();

		Ok(script)
	}

	async fn call_function_returning_string(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
	) -> Result<String, ContractError> {
		let output = self.call_invoke_function(function, params, vec![]).await.unwrap();
		self.throw_if_fault_state(&output).unwrap();

		let item = output.stack[0].clone();
		match item.as_string() {
			Some(s) => Ok(s),
			None => Err(ContractError::UnexpectedReturnType("String".to_string())),
		}
	}

	async fn call_function_returning_int(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
	) -> Result<i32, ContractError> {
		let output = self.call_invoke_function(function, params, vec![]).await.unwrap();
		self.throw_if_fault_state(&output).unwrap();

		let item = output.stack[0].clone();
		match item.as_int() {
			Some(i) => Ok(i as i32),
			None => Err(ContractError::UnexpectedReturnType("Int".to_string())),
		}
	}

	async fn call_function_returning_bool(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
	) -> Result<bool, ContractError> {
		let output = self.call_invoke_function(function, params, vec![]).await.unwrap();
		self.throw_if_fault_state(&output).unwrap();

		let item = output.stack[0].clone();
		match item.as_bool() {
			Some(b) => Ok(b),
			None => Err(ContractError::UnexpectedReturnType("Bool".to_string())),
		}
		// .ok_or_else(|| ContractError::UnexpectedReturnType("Bool".to_string()))
	}

	// Other methods

	async fn call_invoke_function(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
		signers: Vec<Signer>,
	) -> Result<InvocationResult, NeoError> {
		if function.is_empty() {
			return Err(NeoError::from(ContractError::InvalidNeoName(
				"Function cannot be empty".to_string(),
			)))
		}
		NeoRust::instance()
			.invoke_function(&self.script_hash().clone(), function.into(), params, signers)
			.request()
			.await
	}

	fn throw_if_fault_state(&self, output: &InvocationResult) -> Result<(), ContractError> {
		if output.has_state_fault() {
			Err(ContractError::UnexpectedReturnType(output.exception.unwrap()))
		} else {
			Ok(())
		}
	}

	// Other methods like `call_function_returning_xxx`, iterators, etc.
	async fn call_function_returning_script_hash(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
	) -> Result<H160, ContractError> {
		let output = self.call_invoke_function(function, params, vec![]).await.unwrap();
		self.throw_if_fault_state(&output).unwrap();

		let item = &output.stack[0];
		item.as_bytes()
			.as_deref()
			.map(|b| H160::from_script(b))
			.ok_or_else(|| ContractError::UnexpectedReturnType("Script hash".to_string()))
	}

	async fn call_function_returning_iterator<U>(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
		mapper: impl Fn(StackItem) -> Result<U, ContractError>,
	) -> Result<NeoIterator<U>, ContractError> {
		let output =
			self.call_invoke_function(function, params, vec![]).await.unwrap().get_result();
		self.throw_if_fault_state(&output).unwrap();

		let item = &output.stack[0];
		let interface = item
			.as_interop()
			.ok_or_else(|| ContractError::UnexpectedReturnType("Iterator".to_string()))
			.unwrap();

		let session_id = output
			.session_id
			.ok_or(ContractError::InvalidNeoNameServiceRoot("No session ID".to_string()))
			.unwrap();

		Ok(NeoIterator::new(session_id, interface.iterator_id, mapper))
	}

	async fn call_function_and_unwrap_iterator<U>(
		&self,
		function: &str,
		params: Vec<ContractParameter>,
		max_items: usize,
		mapper: impl Fn(StackItem) -> U,
	) -> Result<Vec<U>, ContractError> {
		let script = ScriptBuilder::build_contract_call_and_unwrap_iterator(
			&self.script_hash(),
			function,
			params.iter().filter_map(|p| Some(p)).collect(),
			CallFlags::All,
		)
		.unwrap()
		.build();

		let output = NeoRust::instance()
			.invoke_script(script.script().to_hex(), vec![])
			.request()
			.await
			.unwrap();

		self.throw_if_fault_state(&output).unwrap();

		let items = output.stack[0].as_array().unwrap().into_iter().map(mapper).collect();

		Ok(items)
	}

	fn calc_native_contract_hash(contract_name: &str) -> Result<H160, NeoError> {
		Self::calc_contract_hash(H160::zero(), 0, contract_name)
	}

	fn calc_contract_hash(
		sender: H160,
		nef_checksum: u32,
		contract_name: &str,
	) -> Result<H160, NeoError> {
		let mut script = ScriptBuilder::new()
			.op_code(&[OpCode::Abort])
			.push_data(sender.to_vec())
			.unwrap()
			.push_integer(nef_checksum as i64)
			.unwrap()
			.push_data(contract_name.as_bytes().to_vec())
			.unwrap();

		Ok(H160::from_slice(script.script().as_slice()))
	}

	async fn get_manifest(&self) -> ContractManifest {
		NeoRust::instance()
			.get_contract_state(self.script_hash())
			.request()
			.await
			.unwrap()
			.manifest
	}
}
