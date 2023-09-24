// iterator

use crate::{
	neo_error::NeoError,
	protocol::{
		core::{neo_trait::NeoTrait, stack_item::StackItem},
		neo_rust::NeoRust,
	},
	NEO_INSTANCE,
};
use std::{fmt, sync::Arc};

use crate::protocol::http_service::HttpService;

pub struct NeoIterator<T> {
	session_id: String,
	iterator_id: String,
	mapper: Arc<dyn Fn(StackItem) -> T + Send + Sync>,
}

impl<T> fmt::Debug for NeoIterator<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("NeoIterator")
			.field("session_id", &self.session_id)
			.field("iterator_id", &self.iterator_id)
			// For the mapper, you can decide what to print. Here, we just print a static string.
			.field("mapper", &"<function>")
			.finish()
	}
}

impl<T> NeoIterator<T> {
	pub fn new(
		session_id: String,
		iterator_id: String,
		mapper: Arc<dyn Fn(StackItem) -> T + Send + Sync>,
	) -> Self {
		Self { session_id, iterator_id, mapper }
	}

	pub async fn traverse(&self, count: i32) -> Result<Vec<T>, NeoError> {
		let result = NEO_INSTANCE
			.read()
			.unwrap()
			.traverse_iterator(self.session_id.clone(), self.iterator_id.clone(), count as u32)
			.request()
			.await?;
		let mapped = result.iter().map(|item| (self.mapper)(item.clone())).collect();
		Ok(mapped)
	}

	pub async fn terminate_session(&self) -> Result<(), NeoError> {
		NEO_INSTANCE
			.read()
			.unwrap()
			.terminate_session(&self.session_id)
			.request()
			.await
			.expect("Could not terminate session");
		Ok(())
	}
}
