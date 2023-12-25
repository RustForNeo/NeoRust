// iterator
use crate::error::ContractError;
use neo_providers::{JsonRpcClient, Middleware, Provider};
use neo_types::stack_item::StackItem;
use std::{fmt, sync::Arc};

pub struct NeoIterator<'a, T, P: JsonRpcClient> {
	session_id: String,
	iterator_id: String,
	mapper: Arc<dyn Fn(StackItem) -> T + Send + Sync>,
	provider: Option<&'a Provider<P>>,
}

impl<T, P> fmt::Debug for NeoIterator<T, P> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("NeoIterator")
			.field("session_id", &self.session_id)
			.field("iterator_id", &self.iterator_id)
			// For the mapper, you can decide what to print. Here, we just print a static string.
			.field("mapper", &"<function>")
			.finish()
	}
}

impl<T, P> NeoIterator<T, P> {
	pub fn new(
		session_id: String,
		iterator_id: String,
		mapper: Arc<dyn Fn(StackItem) -> T + Send + Sync>,
		provider: Option<&Provider<P>>,
	) -> Self {
		Self { session_id, iterator_id, mapper, provider }
	}

	pub async fn traverse(&self, count: i32) -> Result<Vec<T>, ContractError> {
		let result = self
			.provider
			.traverse_iterator(self.session_id.clone(), self.iterator_id.clone(), count as u32)
			.request()
			.await?;
		let mapped = result.iter().map(|item| (self.mapper)(item.clone())).collect();
		Ok(mapped)
	}

	pub async fn terminate_session(&self) -> Result<(), ContractError> {
		self.provider
			.terminate_session(&self.session_id)
			.request()
			.await
			.expect("Could not terminate session");
		Ok(())
	}
}
